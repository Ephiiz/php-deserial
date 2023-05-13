


use php::{parse_any};
use tracing::{Level};
use tracing_subscriber::FmtSubscriber;

mod php;
mod util;


fn parse_all(input: Vec<&str>) {
    let mut create = String::from("");
    for line in input {create = format!("{}{}", create, line)}


    loop {
        let t = parse_any(create.as_str());
        if t.is_ok() {
            let p = t.unwrap();
            if p.1 == php::Parsed::Eof {
                break;
            }
            create = p.0.to_owned();
        }
    }
}
fn main() {
    let sub = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(sub).expect("setting default subscriber failed");
    let p = parse_any(r#"a:22:{s:13:"iscsiChapUser";s:0:"";s:10:"apiVersion";s:0:"";s:19:"iscsiMutualChapUser";s:0:"";s:8:"hostname";s:11:"MyTestShare";s:10:"nfsEnabled";b:0;s:11:"sftpEnabled";b:0;s:7:"Volumes";a:0:{}s:7:"os_name";s:0:"";s:7:"version";s:0:"";s:14:"iscsiBlockSize";s:0:"";s:4:"name";s:11:"MyTestShare";s:11:"iscsiTarget";s:0:"";s:8:"hostName";s:11:"MyTestShare";s:10:"afpEnabled";b:0;s:11:"apfsEnabled";b:0;s:6:"format";s:4:"ext4";s:4:"type";s:7:"snapnas";s:7:"isIscsi";i:0;s:9:"shareType";s:3:"nas";s:9:"localUsed";d:0.17975234985351562;s:4:"uuid";s:32:"2c3dbe0e9d3c48fdadd6fa932902a74a";s:11:"usedBySnaps";d:0.014881134033203125;}"#).unwrap();
    println!("{}", p.1);

}
