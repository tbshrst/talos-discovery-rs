# Geekweek 2/2025
## Project
* understand and recreate the discovery service from the Talos Linux project

## Setup dev VM in Openstack
* check this repository out on `openstack-cli`
* create vm: ```xfft --stack-name geekweek-vm --setup xfft/setup.yml```
* use vscode to ssh into vm, credentials can be found in xfft/vars.yml
* build devcontainer

# Dokumentation zum Discovery Protokoll

## Discovery Flow
- Protobuf API fuer Client-Server-Kommunikation (discovery-client (Node) ↔ Discovery Service) (Github)
1. Client sendet HelloRequest, wird mit HelloResponse beantwortet
    - HelloRequest
        - ClusterId: zufaelliger String, der initial mit talosctl fuer alle Nodes des Clusters generiert wird
        - Client Version
    - HelloResponse
        - RedirectMessage (optional): Redirect zu anderem Service Endpoint
        - Client IP: oeffentliche IP des discovery-clients
2. Client sendet direkt nach Nachrichtenaustausch 1) einen WatchRequest, wird vom Server mit Stream von WatchResponse beantwortet. WatchResponse treffen asynchron ein
    - WatchRequest: 
        - ClusterId: bestimmt, welcher Cluster beobachtet werden soll (vermutlich lang genug, damit nicht in angemessener Zeit erratbar/bruteforcebar)
        - Allgemeine Info: nach derzeitigem Verstaendnis bleiben Clustereintraege solange im Service, wie der Cluster existiert, in vielen Szenarien existieren K8s Cluster ziemlich lange und sind somit evtl. ratbar?)
    - WatchResponse:
        - Liste von Affiliates
        - flag, ob Affiliates geloescht wurden oder nicht
        - der erste WatchResponse ist immer als snapshot==true markiert, alle darauffolgenden hingegen nicht mehr (Github)
    - Aenderungen in serverseitiger Affiliateliste (hinzufuegen/loeschen) werden an alle discovery-clients gebroadcasted
        - Verarbeitung im discovery-client in "parseReply" (Github)
3. Client sendet direkt nach Nachrichtenaustausch 2) ein AffiliateUpdate
    - AffiliateUpdateRequest:
        - ClusterId: ClusterId im Klartext (wird vom aufrufenden Clientcode waehrend der initialisierung des "Clientmoduls" gesetzt, kommt aus der Konfiguration, die per talosctl generiert wurde)
        - AffiliateId: AffiliateId im Klartext (wird vom aufrufenden Clientcode waehrend der initialisierung des "Clientmoduls" gesetzt, wird vermutlich zufaellig generiert)
        - AffiliateData: verschluesselte Informationen ueber den Node
        - AffiliateEndpoints: verschluesselte Informationen ueber die verfuegbaren Endpunkte (aus Sicht des Clients)
    - AffiliateUpdateResponse
        - leer, wird vom Client ignoriert
    - Verarbeitung im discovery-client in "refreshData" (Github)
        - AffiliateUpdates werden unter folgenden Bedinungen gesendet (TODO: unvollstaendig?!)
            - nach Zeitintervall TTL/2
            - der aufrufende Clientcode updated die Affiliatekonfiguration
4. Aufrufender Clientcode moechte Node vom Cluster loeschen
    - AffiliateDeleteRequest:
        - ClusterId: Zielcluster
        - AffiliateId: ZielAffiliate
    - AffiliateDeleteResponse
        - leer, wird vom Client ignoriert
    - wird in "refresData" ausgeloest, wenn aufrufender Clientcode beschliesst, den Node aus dem Cluster zu loeschen. Wird durch "DeleteLocalAffiliate" ausgeloest (Github)
    - einzige Stelle in Talos, die "DeleteLocalAffiliate" aufruft: Github
    - kennt man beide Werte eines anderen Nodes, kann man diesen aus dem Cluster loeschen. Beide Strings sind sehr wahrscheinlich aber wieder zu lang, um gueltige Paare bruteforcen zu koennen

## Clientinformationen

### Senden

Der client sendet mit AffiliateUpdate verschluesselte Informationen ueber sich selbst an den Service. Die Konfiguration der Affiliateinformationen passiert in SetLocalData (Github), welche vom Client ("TalosOS") aufgerufen wird. Optional sind weitere Endpunkte konfigurierbar.

Es werden folgende Schritte durchgefuehrt:

 1. Affilliateinformationen werden serialisiert
 ```
Pseudocode:

localAffiliateData = serialise(Affiliate {
    affiliate: GrpcAffiliate { // v1alpha1/client/affililate.proto
        node_id: String,
        addresses: Vec<Vec<u8>>,
        hostname: String,
        nodename: String,
        machine_type: String,
        operating_system: String,
        kubespan: Option<KubeSpan { // v1alpha1/client/affililate.proto
            public_key: String,
            address: Vec<u8>,
            additional_addresses: Option<IPPrefix { // v1alpha1/client/affililate.proto
                ip: Vec<u8>,
                bits: u32,
            }>,
        }>
    },
    endpoints: Vec<GrpcEndpoint {  // v1alpha1/client/affililate.proto
        ip: Vec<u8>,
        port: u32,
    }>
});
```
2. Verschluessele Affiliateinformationen
```
verschluesselt BLOB, keine Klartextinformation assoziiert

// https://pkg.go.dev/crypto/cipher#AEAD
// Seal encrypts and authenticates plaintext, authenticates the
// additional data and appends the result to dst, returning the updated
// slice. The nonce must be NonceSize() bytes long and unique for all
// time, for a given key.
//
// To reuse plaintext's storage for the encrypted output, use plaintext[:0]
// as dst. Otherwise, the remaining capacity of dst must not overlap plaintext.
// dst and additionalData may not overlap.
Seal(dst, nonce, plaintext, additionalData []byte) []byte

client.localAffiliate = client.gcm.Seal(client.localAffiliate, nonce, localAffiliateData, nil)
```
→ IV ist die Nonce, der verschluesselte BLOB wird in client.localAffiliate gespeichert

3. Verschluessle Endpunkte
```
enc_endpoints = encrypt(
    Vec<
        Vec<GrpcEndpoint {  // v1alpha1/client/affililate.proto
            ip: Vec<u8>,
            port: u32,
        }>
    >
)
```
4. Verschluessle optionale Endpunkte (wie 3.)

### Empfangen
Der Client bekommt Aenderungen ueber den Stream mit, den er durch den Aufruf von Watch erhalten hat. Der Stream liefert GrpcWatchResponse Datenstrukturen

1. Der client ignoriert seine eigenen AffilliateUpdates (Affiliate ID Eintrag enthaelt die eigene Affiliate ID)
2. soll Affiliate geloescht werden?
3. die gesendete BLOB wird entschluesselt (enthaelt Affiliate und Endpunktinformationen des Affiliates)
    - Ergebnis wird in client.discoveredAffililates gespeichert
    - symmetrischer Key ist PSK, der durch Erzeugung der Clusterkonfiguration durch talsoctl auf allen Nodes gleich ist

## Talos Konfiguration
Eine Übersicht über die potentiell relevanten Konfigurationsparameter. Diese Parameter sind zumindest im aktuellen Setup für alle Maschinen eines Clusters gleich.
```
// Doku: https://www.talos.dev/v1.9/reference/configuration/v1alpha1/config/#Config.machine
machine:
	token: // ein token, dass aktuell bei allen Maschinen des Clusters gleich ist

// Doku: https://www.talos.dev/v1.9/reference/configuration/v1alpha1/config/#Config.cluster
cluster:
	id: // cluster id
	secret: // shared secret of the cluster
	token: // bootstrap token
```

## Verhalten des Discovery Services
- Discovery Service im Debugmodus
- durchgefuehrte Aktionen
    - Hinzufuegen von 6 Teilnehmern
    - sukzessives Zuruecksetzen von Nodekonfigurationen (Reboot machine + Reset Configuration) → fuehrt zur Reduktion der Subscriber, nicht aber der Affiliates
        - Nach Zuruecksetzen scheint die Node ein Watch aufzurufen
    - Loeschen der registrierten VMs → fuehrt zur Reduktion der Affiliates, sind alle Affiliates eines Clusters geloescht, wird der Cluster ebenfalls geloescht
        -  bisher Trigger nicht verstanden, wann Affiliates tatsaechlich geloescht werden
    - Aufruf des Dockercontainers: `podman run -p 3000:3000 -p 3001:3001 ghcr.io/siderolabs/discovery-service:v1.0.9 --debug`
```
::include(file=discovery_service_log.txt)
```
- es werden offensichtlich Affiliates geloescht, aber der Github Code wirkt, als waere der API Call AffilliateDelete implementiert, aber wird nirgends aufgerufen (Github) → AffiliateDelete wird einzig vom discovery-client (Node) aufgerufen (Github)
- aehnlich sieht es beim API Call List aus (Github) → der discovery-client (Node) implementiert nicht den API Call List, tatsaechlich unimplemented? (Github)
- Garbage Collection: Im AffiliateUpdate Request ist ein TTL Wert. Dieser wird vom Server als expiration Date interpretiert. Per Default ist dies einmal jede Mitute (Github)
- Falls der Garbage Collector einen affiliate findet, der expired ist, wird dieser gelöscht. Das scheint im Endeffekt dann der gleiche Codepfad zu sein wie der DeleteAffiliate API-Call (Github)

## Links
öffentliche Instanz: https://discovery.talos.dev/

Doku: https://www.talos.dev/v1.10/talos-guides/discovery/

Repos:

- Server: https://github.com/siderolabs/discovery-service (v1.0.10)
- Client: https://github.com/siderolabs/discovery-client (v0.1.11)
- API: https://github.com/siderolabs/discovery-api/ (v0.1.6)
