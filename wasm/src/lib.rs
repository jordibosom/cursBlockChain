// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            7
// Async Callback (empty):               1
// Total number of exported functions:   9

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    projecte_institut
    (
        init => init
        afegirTipusItem => afegir_tipus_item
        solicitarPrestec => solicitar_prestec
        aprovarPrestec => aprovar_prestec
        confirmarRetorn => confirmar_retorn
        consultarPrestec => consultar_prestec
        consultarDisponibles => consultar_disponibles
        consultarSolicitudPendent => consultar_solicitud_pendent
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
