/// Portal

use crate::ErrorCode;

pub trait Portal<'a, Traveler> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<Traveler=Traveler>);

    fn teleport(
        &self,
        traveler: &'static mut Traveler,
    ) -> Result<(), (ErrorCode, &'static mut Traveler)>;
}

pub trait PortalClient {
    type Traveler;

    fn teleported(
        &self,
        traveler: &'static mut Self::Traveler,
    ) -> Result<(), (ErrorCode, &'static mut Self::Traveler)>;
}
