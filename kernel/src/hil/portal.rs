/// Portal

use crate::ErrorCode;

pub trait Portal<'a, Traveler> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<Traveler>);

    fn teleport(
        &self,
        traveler: &'static mut Traveler,
    ) -> Result<(), (ErrorCode, &'static mut Traveler)>;
}

pub trait PortalClient<Traveler> {
    fn teleported(
        &self,
        traveler: &'static mut Traveler,
        rcode: Result<(), ErrorCode>,
    );
}
