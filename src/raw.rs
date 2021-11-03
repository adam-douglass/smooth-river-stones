use web_sys::Node;
use yew::{Component, ComponentLink, Html, Properties, ShouldRender, virtual_dom::VNode};

#[derive(Debug, Clone, Eq, PartialEq, Properties)]
pub struct RawProps {
    pub inner_html: String,
}

pub struct Raw {
    pub props: RawProps,
}

impl Component for Raw {
    type Message = ();
    type Properties = RawProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let span = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("span")
            .unwrap();
        span.set_class_name("content");
        span.set_inner_html(&self.props.inner_html[..]);

        let node = Node::from(span);
        let vnode = VNode::VRef(node);
        vnode
    }
}