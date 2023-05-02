#[derive(Eq)]
pub(crate) struct Task
{
    priority: usize,
}

impl Task
{
    pub(crate) fn new() -> Self
    {
        Self
        {
            priority: 0,
        }
    }
}

impl std::cmp::Ord for Task
{
    fn cmp( &self, other: &Self ) -> std::cmp::Ordering
    {
        self.priority.cmp(&other.priority)
    }
}

impl std::cmp::PartialOrd for Task
{
    fn partial_cmp( &self, other: &Self ) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for Task
{
    fn eq( &self, other: &Self ) -> bool
    {
        self.priority == other.priority
    }
}
