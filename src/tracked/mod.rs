use std::io;

pub trait Version {
    fn save(&mut self) -> io::Result<()>;

    fn load(&mut self, index: usize) -> io::Result<()>;

    fn delete(&mut self, index: usize) -> io::Result<()>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn load_last(&mut self) -> io::Result<()> {
        self.load(self.len().saturating_sub(1))
    }

    fn delete_last(&mut self) -> io::Result<()> {
        self.delete(self.len().saturating_sub(1))
    }

    fn reset(&mut self) -> io::Result<()> {
        self.load_last()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.delete(0)
    }

    fn split(&mut self, index: usize) -> io::Result<Self> where Self: Sized + Clone {
        self.load(index)?;
        let mut other = self.clone();
        other.clear()?;
        other.save()?;
        Ok(other)
    }
}
