use pasture_core::containers::VectorBuffer;

#[derive(Debug)]
pub enum WriteQuery {
    Remove,
    Add(VectorBuffer),
    Update(VectorBuffer),
}