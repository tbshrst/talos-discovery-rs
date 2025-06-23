# Anforderungen auf S/P-Client
Aus mitarbeiter.genua.de -> Client -> S-Client-Software

- Buildmanagement make
- Pandoc Parser fuer Dokumente
- Textsatzsystem TerX Live

# Beispiel als submodul und mittels make(1)

```sh
git submodule add ssh://git@gitlab.genua.de/carsten/g20beamer.git
```

## Slides

```Makefile
LATEX=xelatex
PANDOC=pandoc

%.pdf: %.md
	TTFONTS=$$TTFONTS:./g20beamer \
	TEXINPUTS=$$TEXINPUTS:./g20beamer \
	${PANDOC} -o $@ -s --template ./g20beamer/g24beamer.pandoc.template -t beamer $< --listings --slide-level 2 --pdf-engine=${LATEX}
```

## Slides in Quarto und Pandoc

```yaml
---
title: Titel des Vortrags
isms: <genua-ISMS-Einstufung>
socials: true
format:
  beamer:
    template: g24beamer.pandoc.template 
---
```

## Fliesstext

```Makefile
LATEX=xelatex
PANDOC=pandoc

%.pdf: %.md
	TTFONTS=$$TTFONTS:./g20beamer \
	TEXINPUTS=$$TEXINPUTS:./g20beamer \
	${PANDOC} -o $@ -s --template ./g20beamer/g.template $< --pdf-engine=${LATEX} --highlight-style tango
```

# Known-Bugs

- lualatex hat evtl Probleme mit Font-Caching -> xelatex
- Fliesstext kann noch keine Titelseite
