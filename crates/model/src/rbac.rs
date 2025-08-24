pub struct ResourceDescriptor {
    pub path: String,
}


/// For every resource
pub enum Permission {
    // Create resource
    Create,
    // Delete resource
    Delete,
    // Update resource
    Update,
    // Retrieve resource
    Retrieve,
    // 
    Reload,
    Rollback,
}
