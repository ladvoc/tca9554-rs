use embedded_hal_async::i2c::{ErrorKind, NoAcknowledgeSource};
use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
use tca9554::{Address, Tca9554};

#[tokio::test]
async fn test_write_direction() {
    let i2c = Mock::new(&[
        Transaction::write(0x20, vec![0x03, 0xAA])
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    driver.write_direction(0xAA).await.unwrap();
    driver.release().done();
}

#[tokio::test]
async fn test_write_output() {
    let i2c = Mock::new(&[
        Transaction::write(0x20, vec![0x01, 0xAA])
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    driver.write_output(0xAA).await.unwrap();
    driver.release().done();
}

#[tokio::test]
async fn test_write_polarity() {
    let i2c = Mock::new(&[
        Transaction::write(0x20, vec![0x02, 0xAA])
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    driver.write_polarity(0xAA).await.unwrap();
    driver.release().done();
}

#[tokio::test]
async fn test_write_error() {
    let no_ack = ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address);
    let i2c = Mock::new(&[
        Transaction::write(0x20, vec![0x03, 0xAA]).with_error(no_ack)
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    assert!(driver.write_direction(0xAA).await.is_err());
    driver.release().done();
}

#[tokio::test]
async fn test_read_input() {
    let i2c = Mock::new(&[
        Transaction::write_read(0x20, vec![0x00], vec![0xAA])
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    assert_eq!(driver.read_input().await.unwrap(), 0xAA);
    driver.release().done();
}

#[tokio::test]
async fn test_reset() {
    let i2c = Mock::new(&[
        Transaction::write(0x20, vec![0x03, 0xFF]),
        Transaction::write(0x20, vec![0x02, 0x00]),
        Transaction::write(0x20, vec![0x01, 0xFF])
    ]);
    let mut driver = Tca9554::new(i2c, Address::standard());
    driver.reset().await.unwrap();
    driver.release().done();
}