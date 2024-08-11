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
        product_id: u32,
        blueprint: Item, // runs are ignored
        kind: ManufacturingKind,
        system_id: u32, // ignored if include_security is false
        include: DatabaseParamsInclude,
    ) -> Result<DatabaseResponse, Self::Error>;
    async fn get_volume(
        &self,
        item: u32,
    ) -> Result<Option<Volume>, Self::Error>;
    async fn get_name(&self, item: u32) -> Result<String, Self::Error>;
}

impl<T> IndustryDatabase for T
where
    T: InnerDatabase,
{
    async fn compute_line(
        &self,
        // location config
        system_id: u32,
        structure_id: u32,
        rigs: [Option<u32>; 3],
        tax: config::ManufacturingValue,
        // config
        skills: &HashMap<u32, u8>,
        // production line
        kind: config::ManufacturingKind,
        transput: config::Transput,
        max_duration: Duration,
        decryptor: Option<u32>,
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
    async fn get_volume(
        &self,
        item: Item,
    ) -> Result<Option<Volume>, crate::Error> {
        self.get_volume(item.type_id)
            .await
            .map_err(|e| crate::Error::IndustryDbError(e.into()))
    }
    async fn get_name(&self, item: Item) -> Result<String, crate::Error> {
        let name = self
            .get_name(item.type_id)
            .await
            .map_err(|e| crate::Error::IndustryDbError(e.into()))?;
        Ok(match item.is_blueprint() {
            true => format!(
                "{} (me: {}, te: {}, runs: {})",
                name, item.me, item.te, item.runs
            ),
            false => name,
        })
    }
}
