use capsules::permissions::Permissions;
use kernel::component::Component;
use kernel::static_init;

pub struct PermissionsComponent {}

impl PermissionsComponent {
    pub fn new() -> PermissionsComponent {
        PermissionsComponent {}
    }
}

impl Component for PermissionsComponent {
    type Output = &'static Permissions;

    unsafe fn finalize(&mut self) -> Self::Output {
        static_init!(Permissions, Permissions::new())
    }
}
