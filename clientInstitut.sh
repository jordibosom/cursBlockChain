#!/bin/bash

################################################################################
# CONFIGURACIÓ - MODIFICA AQUESTES VARIABLES
################################################################################

# Adreça del Smart Contract un cop desplegat
CONTRACT_ADDRESS="erd1qqqqqqqqqqqqqpgquuqezkwswma3x9tegn548mmxa5zaa3vd6zfsq52rcd"

# Camí al fitxer PEM de l'administrador (institut)
PEM_ADMIN="./wallet.pem"
# Adreça de l'administrador
ADDRESS_ADMIN=$(mxpy wallet pem-address $PEM_ADMIN)


# Camí al fitxer PEM d'un alumne d'exemple
PEM_ALUMNE="./wallets/alumne1.pem"
# Adreça de l'alumne
ADDRESS_ALUMNE=$(mxpy wallet pem-address $PEM_ALUMNE)

# Configuració de la xarxa (devnet, testnet o mainnet)
PROXY="https://devnet-gateway.multiversx.com"
CHAIN_ID="D"

# Límits de gas
GAS_LIMIT=10000000

################################################################################
# FUNCIONS AUXILIARS
################################################################################

# Funció per executar una transacció (crida a un endpoint)
# Ús: executar_tx <funcio> <pem_file> [arguments...]
executar_tx() {
    local funcio=$1
    local pem_file=$2
    shift 2
    local args=("$@")

    echo "Executant transacció..."
    mxpy --verbose contract call ${CONTRACT_ADDRESS} --recall-nonce --pem=${pem_file} \
        --gas-limit=${GAS_LIMIT} --proxy=${PROXY} --chain=${CHAIN_ID} \
        --function="${funcio}" --arguments "${args[@]}" --send || echo "ERROR: La transacció ha fallat."
}

# Funció per executar una consulta (crida a una vista)
# Ús: executar_query <funcio> [arguments...]
executar_query() {
    local funcio=$1
    shift
    local args=("$@")

    echo "Executant consulta..."
    mxpy --verbose contract query ${CONTRACT_ADDRESS} --proxy=${PROXY} \
        --function="${funcio}" --arguments "${args[@]}"
}

################################################################################
# MENÚ PRINCIPAL
################################################################################

while true; do
    echo "=================================================="
    echo "    SISTEMA DE GESTIÓ DE PRÉSTECS DE L'INSTITUT    "
    echo "=================================================="
    echo
    echo "--- MENÚ ADMINISTRADOR ---"
    echo "  1. Afegir tipus d'item a l'inventari"
    echo "  2. Aprovar un préstec pendent"
    echo "  3. Confirmar el retorn d'un article"
    echo
    echo "--- MENÚ ALUMNE ---"
    echo "  4. Sol·licitar un préstec"
    echo
    echo "--- CONSULTES GENERALS ---"
    echo "  5. Consultar inventari disponible"
    echo "  6. Consultar estat d'un préstec actiu"
    echo "  7. Consultar una sol·licitud pendent"
    echo
    echo "  0. Sortir"
    echo "--------------------------------------------------"
    read -p "Tria una opció: " opcio

    case $opcio in
        1)
            read -p "Introdueix el tipus d'item (ex: portatil): " tipus
            read -p "Introdueix la quantitat inicial: " quantitat
            executar_tx "afegirTipusItem" "${PEM_ADMIN}" "str:${tipus}" "${quantitat}"
            ;;
        2)
            read -p "Introdueix l'adreça de l'alumne a aprovar (erd1...): " adreca_alumne
            executar_tx "aprovarPrestec" "${PEM_ADMIN}" "addr:${adreca_alumne}"
            ;;
        3)
            read -p "Introdueix l'adreça de l'alumne que retorna (erd1...): " adreca_alumne
            executar_tx "confirmarRetorn" "${PEM_ADMIN}" "addr:${adreca_alumne}"
            ;;
        4)
            read -p "Introdueix el tipus d'item a sol·licitar (ex: portatil): " tipus
            echo "Es farà la sol·licitud com a ALUMNE: ${ADDRESS_ALUMNE}"
            executar_tx "solicitarPrestec" "${PEM_ALUMNE}" "str:${tipus}"
            ;;
        5)
            read -p "De quin tipus d'item vols consultar la disponibilitat? " tipus
            executar_query "consultarDisponibles" "str:${tipus}"
            ;;
        6)
            read -p "De quina adreça vols consultar el préstec actiu? " adreca
            executar_query "consultarPrestec" "addr:${adreca}"
            ;;
        7)
            read -p "De quina adreça vols consultar la sol·licitud pendent? " adreca
            executar_query "consultarSolicitudPendent" "addr:${adreca}"
            ;;
        0)
            echo "Sortint..."
            break
            ;;
        *)
            echo "Opció invàlida. Torna a intentar-ho."
            ;;
    esac
    echo
    read -p "Prem [Enter] per continuar..."
    clear
done