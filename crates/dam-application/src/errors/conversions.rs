use super::*;

impl From<ScheduleError> for UseCaseError {
    fn from(e: ScheduleError) -> UseCaseError {
        UseCaseError::Schedule(e)
    }
}
impl From<Refusal> for UseCaseError {
    fn from(r: Refusal) -> UseCaseError {
        UseCaseError::Refused(r)
    }
}
impl From<ItemRefusal> for UseCaseError {
    fn from(r: ItemRefusal) -> UseCaseError {
        UseCaseError::Refused(Refusal::Item(r))
    }
}
impl From<StoreError> for UseCaseError {
    fn from(e: StoreError) -> UseCaseError {
        UseCaseError::Store(e)
    }
}
impl From<HelperError> for UseCaseError {
    fn from(e: HelperError) -> UseCaseError {
        UseCaseError::Helper(e)
    }
}
impl From<CredentialError> for UseCaseError {
    fn from(e: CredentialError) -> UseCaseError {
        UseCaseError::Credential(e)
    }
}
