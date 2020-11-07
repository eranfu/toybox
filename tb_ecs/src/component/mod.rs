use storage::Storage;

mod storage;

trait Component {
    type Storage: Storage;
}