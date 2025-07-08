#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

/// Aquest contracte gestiona el préstec de material informàtic per a un institut,
/// separant les responsabilitats entre un rol d'Administrador (`owner`) i els Usuaris (alumnes).
/// El cicle de vida d'un préstec és: Sol·licitud -> Aprovació -> Retorn Confirmat.

// --- Estructures de Dades ---

/// Conté la informació essencial d'un préstec que ja ha estat aprovat i està actiu.
#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct PrestecInfo<M: ManagedTypeApi> {
    /// El tipus d'article prestat (p.ex., "portatil", "teclat").
    pub tipus_item: ManagedBuffer<M>,
    /// La data límit de retorn, guardada com a timestamp de bloc (en segons).
    pub data_retorn: u64,
}

/// Defineix la interfície del contracte amb tots els seus endpoints i vistes públiques.
#[multiversx_sc::contract]
pub trait PrestecsInstitut {
    /// Funció d'inicialització. S'executa només un cop en desplegar el contracte.
    #[init]
    fn init(&self) {
        // Obtenim l'adreça de la wallet que desplega el contracte.
        let caller = self.blockchain().get_caller();
        // La guardem com a propietària (administrador) del contracte.
        // Això garanteix que només qui crea el contracte en té el control inicial.
        self.owner().set(caller);
    }

    // ===================================================================
    // === Endpoints d'Administrador (requereixen ser el 'owner') ===
    // ===================================================================

    /// Endpoint privat per afegir un nou tipus d'article a l'inventari o actualitzar-ne la quantitat.
    /// Només pot ser cridat per l'administrador (`owner`).
    #[only_owner]
    #[endpoint(afegirTipusItem)]
    fn afegir_tipus_item(&self, tipus_item: ManagedBuffer, quantitat_inicial: usize) {
        // Validació: La quantitat inicial ha de ser superior a zero per evitar estats invàlids.
        require!(
            quantitat_inicial > 0,
            "La quantitat ha de ser més gran que 0"
        );

        // Donem d'alta el nou tipus d'article, establint el seu estoc total i disponible.
        self.inventari_total(&tipus_item).set(quantitat_inicial);
        self.inventari_disponible(&tipus_item)
            .set(quantitat_inicial);
    }

    /// Endpoint privat que permet a l'administrador aprovar una sol·licitud de préstec pendent.
    #[only_owner]
    #[endpoint(aprovarPrestec)]
    fn aprovar_prestec(&self, adreca_usuari: ManagedAddress) {
        // Validació 1: Comprovem que l'usuari realment té una sol·licitud pendent.
        require!(
            !self.solicituds_pendents(&adreca_usuari).is_empty(),
            "Aquest usuari no té cap sol·licitud pendent."
        );

        let tipus_item = self.solicituds_pendents(&adreca_usuari).get();

        // Validació 2: Comprovem que hi ha unitats disponibles en el moment de l'aprovació.
        let mut unitats_disponibles = self.inventari_disponible(&tipus_item).get();
        require!(
            unitats_disponibles > 0,
            "No queden unitats disponibles d'aquest article."
        );

        // --- Accions (Transició d'Estat: de 'Pendent' a 'Actiu') ---

        // 1. Eliminem la sol·licitud de la llista de pendents.
        self.solicituds_pendents(&adreca_usuari).clear();

        // 2. Actualitzem l'inventari restant una unitat.
        unitats_disponibles -= 1;
        self.inventari_disponible(&tipus_item)
            .set(unitats_disponibles);

        // 3. Calculem la data de retorn (p.ex., 15 dies des d'ara).
        let periode_prestec_segons = 15 * 24 * 60 * 60; // 15 dies en segons
        let data_retorn = self.blockchain().get_block_timestamp() + periode_prestec_segons;

        // 4. Creem la "fitxa" del préstec i la guardem al registre de préstecs actius.
        let info_prestec = PrestecInfo {
            tipus_item,
            data_retorn,
        };
        self.prestecs(&adreca_usuari).set(info_prestec);
    }

    /// Endpoint privat que permet a l'administrador confirmar la recepció d'un article retornat.
    #[only_owner]
    #[endpoint(confirmarRetorn)]
    fn confirmar_retorn(&self, adreca_usuari: ManagedAddress) {
        // Validació: Comprovem que l'usuari tenia un préstec actiu per retornar.
        require!(
            !self.prestecs(&adreca_usuari).is_empty(),
            "Aquest usuari no té cap article en préstec per retornar."
        );

        // Obtenim la informació del préstec per saber quin tipus d'article hem de reincorporar a l'inventari.
        let info_prestec = self.prestecs(&adreca_usuari).get();
        let tipus_item_retornat = info_prestec.tipus_item;

        // --- Accions (Transició d'Estat: de 'Actiu' a 'Retornat') ---

        // 1. Eliminem el préstec del registre d'actius.
        self.prestecs(&adreca_usuari).clear();

        // 2. Incrementem les unitats disponibles de l'article retornat.
        let mut unitats_disponibles = self.inventari_disponible(&tipus_item_retornat).get();
        unitats_disponibles += 1;
        self.inventari_disponible(&tipus_item_retornat)
            .set(unitats_disponibles);
    }

    // ===================================================================
    // === Endpoints d'Usuari (Alumnes) ===
    // ===================================================================

    /// Endpoint públic que permet a un usuari (alumne) sol·licitar un article.
    /// La sol·licitud queda en estat pendent fins que l'administrador l'aprovi.
    #[endpoint(solicitarPrestec)]
    fn solicitar_prestec(&self, tipus_item: ManagedBuffer) {
        // Obtenim l'adreça de qui fa la crida.
        let caller = self.blockchain().get_caller();

        // --- Validacions (Regles de Negoci) ---
        // 1. L'usuari no pot demanar un article si ja en té un altre en préstec actiu.
        require!(
            self.prestecs(&caller).is_empty(),
            "Ja tens un article en préstec actiu."
        );
        // 2. L'usuari no pot fer una nova sol·licitud si ja en té una de pendent.
        require!(
            self.solicituds_pendents(&caller).is_empty(),
            "Ja tens una sol·licitud pendent."
        );
        // 3. El tipus d'article sol·licitat ha d'existir a l'inventari.
        require!(
            !self.inventari_total(&tipus_item).is_empty(),
            "Aquest tipus d'article no existeix."
        );

        // Si totes les validacions són correctes, registrem la sol·licitud.
        self.solicituds_pendents(&caller).set(tipus_item);
    }

    // ===================================================================
    // === Vistes (Consultes de Només Lectura) ===
    // ===================================================================

    /// Retorna la informació del préstec actiu d'una adreça específica.
    #[view(consultarPrestec)]
    fn consultar_prestec(&self, adreca: ManagedAddress) -> OptionalValue<PrestecInfo<Self::Api>> {
        let prestec = self.prestecs(&adreca);
        if prestec.is_empty() {
            OptionalValue::None // Retorna 'None' si l'usuari no té cap préstec actiu.
        } else {
            OptionalValue::Some(prestec.get()) // Retorna la informació del préstec si en té.
        }
    }

    /// Retorna la quantitat d'unitats disponibles d'un tipus d'article.
    #[view(consultarDisponibles)]
    fn consultar_disponibles(&self, tipus_item: ManagedBuffer) -> usize {
        self.inventari_disponible(&tipus_item).get()
    }

    /// Retorna el tipus d'article que ha sol·licitat un usuari i que està pendent d'aprovació.
    #[view(consultarSolicitudPendent)]
    fn consultar_solicitud_pendent(&self, adreca: ManagedAddress) -> OptionalValue<ManagedBuffer> {
        let solicitud = self.solicituds_pendents(&adreca);
        if solicitud.is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(solicitud.get())
        }
    }

    // --- Storage Mappers ---
    // Aquesta secció defineix les "variables" o "taules" on el contracte
    // emmagatzema permanentment el seu estat a la blockchain.

    /// Guarda l'adreça de l'administrador del contracte.
    #[storage_mapper("owner")]
    fn owner(&self) -> SingleValueMapper<ManagedAddress>;

    /// Mapa que associa un tipus d'item amb la seva quantitat total registrada.
    #[storage_mapper("inventariTotal")]
    fn inventari_total(&self, tipus_item: &ManagedBuffer) -> SingleValueMapper<usize>;

    /// Mapa que associa un tipus d'item amb la quantitat actualment disponible per a préstec.
    #[storage_mapper("inventariDisponible")]
    fn inventari_disponible(&self, tipus_item: &ManagedBuffer) -> SingleValueMapper<usize>;

    /// Mapa que associa l'adreça d'un usuari amb la informació del seu préstec ACTIU.
    #[storage_mapper("prestecs")]
    fn prestecs(&self, adreca_usuari: &ManagedAddress)
        -> SingleValueMapper<PrestecInfo<Self::Api>>;

    /// Mapa que associa l'adreça d'un usuari amb la seva sol·licitud de préstec PENDENT.
    #[storage_mapper("solicitudsPendents")]
    fn solicituds_pendents(
        &self,
        adreca_usuari: &ManagedAddress,
    ) -> SingleValueMapper<ManagedBuffer>;
}
