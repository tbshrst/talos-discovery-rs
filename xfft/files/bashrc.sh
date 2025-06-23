docker_clean()
{
    image_ids=$(docker images --filter "dangling=true" --quiet)
    docker rmi -f $image_ids
}

info()
{
    local NC='\033[0m'
    local BLUE='\033[0;34m'

    printf "[${BLUE}INFO${NC}] $1\n"
}

dev()
{
    local rebuild_image
    local restart_container
    local opt OPTIND
    local ALL_CONTAINERS
    local CONTAINER
    local STATUS

    while getopts "ifc" opt; do
        case $opt in
            i)
                rebuild_image=1
                ;;
            f)
                no_cache=1
                ;;
            c)
                restart_container=1
                ;;
            *)
                info "unknown parameter"
                info ""
                info "-i    rebuild image"
                info "-f    ignore docker cache"
                info "-c    restart container"
                return 1
                ;;
        esac
    done
    shift $((OPTIND - 1))

    if [ -z "$(which docker)" ]
    then
        info "Docker needs to be installed"
        return 1
    fi

    if [ -z "$(docker image ls | grep "gv-dev")" ] || [ "$rebuild_image" == "1" ]
    then
        if [ "$rebuild_image" == "1" ]; then
            info "rebuilding image.."
        else
            info "no devcontainer found, building image.."
        fi
        echo -n "enter path to repository (default: ~/geekweek): "
        read REPO_PATH
        REPO_PATH=${REPO_PATH:-~/geekweek}
        [ "$no_cache" == "1" ] && BUILD_ARG="--no-cache"
        docker buildx build $BUILD_ARG -t gv-dev:latest -f $REPO_PATH/.devcontainer/Dockerfile .
        if [ $? != 0 ]
        then
            info "build failed"
            return 1
        fi
    fi

    if [ "$restart_container" == "1" ]
    then
        info "killing container.."
        docker rm -f gv-dev &> /dev/null
    fi

    ALL_CONTAINERS=$(docker ps -a)
    if [ "$(echo $ALL_CONTAINERS | grep -m1 "gv-dev")" ]
    then
        CONTAINER_ENTRY="$(echo "$ALL_CONTAINERS" | grep -m1 "gv-dev")"
    else
        info "starting new container"
            docker run -d --name gv-dev \
            --network=host \
            -v ~:/workspace \
            $([ -d ~/.kube ] && echo "-v ~/.kube:/root/.kube") \
            \#$([ -S /var/run/docker.sock ] && echo "-v /var/run/docker.sock:/var/run/user/0/container.sock") \
            \#$([ -S /var/run/docker.sock ] && echo "-e DOCKER_HOST=unix:///var/run/user/0/container.sock") \
            \#$([ -S $XDG_RUNTIME_DIR/docker.sock ] && echo "-v /var/run/docker.sock:/var/run/user/0/container.sock") \
            \#$([ -S $XDG_RUNTIME_DIR/docker.sock ] && echo "-e DOCKER_HOST=unix:///var/run/user/0/container.sock") \
            $([ -S $XDG_RUNTIME_DIR/podman/podman.sock ] && echo "-v $XDG_RUNTIME_DIR/podman/podman.sock:/var/run/user/0/container.sock") \
            $([ -S $XDG_RUNTIME_DIR/podman/podman.sock ] && echo "-e CONTAINER_HOST=unix:///var/run/user/0/container.sock") \
            $([ -S "$SSH_AUTH_SOCK" ] && echo "-v $SSH_AUTH_SOCK:/var/run/agent.sock") \
            $([ -S "$SSH_AUTH_SOCK" ] && echo "-e SSH_AUTH_SOCK=/var/run/agent.sock") \
            -e http_proxy=$HTTP_PROXY \
            -e HTTP_PROXY=$HTTP_PROXY \
            -e https_proxy=$HTTPS_PROXY \
            -e HTTPS_PROXY=$HTTPS_PROXY \
            -e no_proxy=$NO_PROXY \
            -e NO_PROXY=$NO_PROXY \
            gv-dev:latest sleep inf
        CONTAINER_ENTRY="$(docker ps -a | grep -m1 "gv-dev")"
    fi

    CONTAINER_ID=$(echo "$CONTAINER_ENTRY" | awk '{print $1}')
    STATUS=$(docker inspect --format='{{.State.Status}}' "$CONTAINER_ID")
    if [ $? == "1" ]
    then
        info "container doesn't exist"
        return 1
    fi

    if [ "$STATUS" == "exited" ] || [ "$STATUS" == "paused" ]
    then
        info "starting stopped container"
        docker start $CONTAINER_ID
    elif [ "$STATUS" == "restarting" ]
    then
        while [ "$STATUS" != "running" ]
        do
            echo "waiting for container.."
            sleep 2
            STATUS=$(docker inspect --format='{{.State.Status}}' "$CONTAINER_ID")
        done
    fi

    docker exec -it $CONTAINER_ID bash
}
