

// pub trait MapM {
//     pub fn map<B>(
//         self,
//         f: impl Fn(Message) -> B + 'a,
//     ) -> Element<'a, B, Theme, Renderer>
//     where
//         Message: 'a,
//         Theme: 'a,
//         Renderer: crate::Renderer + 'a,
//         B: 'a,
//     {
//         Element::new(Map::new(self.widget, f))
//     }
// }