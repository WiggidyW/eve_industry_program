use super::*;

pub struct DatabaseParamsInclude {
    pub minerals: bool,
    pub installation_minerals: bool,
    pub efficiencies: bool,
    pub security: bool, // if this is false, then system_id parameter is ignored
}

pub trait InnerDatabase: Send + Sync {
    type Error: Into<Box<dyn std::error::Error>>;
    async fn get(
        &self,
        product_id: TypeId,
        blueprint: Item, // runs are ignored
        kind: ManufacturingKind,
        system_id: SystemId, // ignored if include_security is false
        include: DatabaseParamsInclude,
    ) -> Result<DatabaseResponse, Self::Error>;
    async fn get_volume(&self, item: TypeId) -> Result<Volume, Self::Error>;
}

impl<T> IndustryDatabase for T
where
    T: InnerDatabase,
{
    async fn compute_line(
        &self,
        // location config
        system_id: SystemId,
        structure_id: TypeId,
        rigs: [Option<TypeId>; 3],
        tax: config::ManufacturingValue,
        // config
        skills: &HashMap<u32, u8>,
        // production line
        kind: config::ManufacturingKind,
        transput: config::Transput,
        max_duration: Duration,
        decryptor: Option<TypeId>,
    ) -> Result<Line, crate::Error> {
        let rep = self
            .get(
                transput.product.type_id,
                transput.blueprint,
                kind,
                system_id,
                Line::database_params_include(),
            )
            .await
            .map_err(|e| crate::Error::IndustryDbError(e.into()))?;
        Line::from_rep(
            rep,
            structure_id,
            rigs,
            tax,
            skills,
            kind,
            transput,
            max_duration,
            decryptor,
        )
    }
    async fn get_volume(&self, item: Item) -> Result<Volume, crate::Error> {
        self.get_volume(item.type_id)
            .await
            .map_err(|e| crate::Error::IndustryDbError(e.into()))
    }
}
