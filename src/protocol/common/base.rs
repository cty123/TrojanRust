// macro_rules! impl_foo {
//     ($($t:ty),+) => {
//         $(impl Foo for $t {
//             fn foo(&self) {
//                 // Implementation code here
//                 println!("{:?}", self.x);
//             }
//         })+
//     }
// }
//
// impl_foo!(A, B);