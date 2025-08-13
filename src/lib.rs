#![allow(static_mut_refs)]

use std::{cell::OnceCell, sync::Arc};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::{Function, Reflect}, window, MessageEvent, RtcDataChannelEvent, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSessionDescriptionInit};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

macro_rules! console_log {
($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let w = window().expect("can't get window, probably not running in a browser");
    let d: web_sys::HtmlDocument = {
        let temp_d = w.document().unwrap();
        temp_d.dyn_into::<web_sys::HtmlDocument>()?
    };

    let start_b: web_sys::HtmlElement = d.create_element("button")?.dyn_into()?;
    start_b.set_onclick(Some(&Function::new_no_args("start_connect()")));
    start_b.set_id("start_button");
    start_b.set_text_content(Some("start connection"));
    d.body().unwrap().append_child(&start_b)?;

    let connect_b: web_sys::HtmlElement = d.create_element("button")?.dyn_into()?;
    connect_b.set_onclick(Some(&Function::new_no_args("respond_connect()")));
    connect_b.set_id("connect_button");
    connect_b.set_text_content(Some("respond to connection"));
    d.body().unwrap().append_child(&connect_b)?;

    Ok(())
}

static mut PEER_CONN: OnceCell<Arc<RtcPeerConnection>> = OnceCell::new();

#[wasm_bindgen]
pub async fn start_connect() -> Result<(), JsValue> {
    log("start connecting");

    let peer_conn =
        unsafe { PEER_CONN.get_or_init(|| Arc::new(RtcPeerConnection::new().unwrap())) };
    let p2 = RtcPeerConnection::new()?;

    let data1_channel = peer_conn.create_data_channel("hnefatafl");

    let d1_clone = data1_channel.clone();
    let on_msg_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
        if let Some(msg) = event.data().as_string() {
            log(&format!("{:?}", msg));
            d1_clone.send_with_str("ping").unwrap();
        }
    });
    data1_channel.set_onmessage(Some(on_msg_callback.as_ref().unchecked_ref()));
    // hand it over to the js_gc so it will get kept while needed
    on_msg_callback.forget();

    let on_channel_callback = Closure::<dyn FnMut(_)>::new(move |ev: RtcDataChannelEvent| {
        let dc2 = ev.channel();
        log(&format!("pc2.onchannel: {:?}", dc2.label()));

        let on_message_callback = Closure::<dyn FnMut(_)>::new(move |ev: MessageEvent| {
            if let Some(message) = ev.data().as_string() {
                log(&format!("{:?}", message));
            }
        });
        dc2.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
        on_message_callback.forget();


        let dc2_clone = dc2.clone();
        let on_open_callback = Closure::<dyn FnMut()>::new(move || {
            dc2_clone.send_with_str("ping from dc2").unwrap();
        });
        dc2.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));
        on_open_callback.forget();
    });
    p2.set_ondatachannel(Some(on_channel_callback.as_ref().unchecked_ref()));
    on_channel_callback.forget();

   /*
     * Handle ICE candidate each other
     *
     */
    let pc2_clone = p2.clone();
    let onicecandidate_callback1 =
        Closure::<dyn FnMut(_)>::new(move |ev: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = ev.candidate() {
                console_log!("pc1.onicecandidate: {:#?}", candidate.candidate());
                let _ = pc2_clone.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate));
            }
        });
    peer_conn.set_onicecandidate(Some(onicecandidate_callback1.as_ref().unchecked_ref()));
    onicecandidate_callback1.forget();

    let pc1_clone = peer_conn.clone();
    let onicecandidate_callback2 =
        Closure::<dyn FnMut(_)>::new(move |ev: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = ev.candidate() {
                console_log!("pc2.onicecandidate: {:#?}", candidate.candidate());
                let _ = pc1_clone.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate));
            }
        });
    p2.set_onicecandidate(Some(onicecandidate_callback2.as_ref().unchecked_ref()));
    onicecandidate_callback2.forget();

    let offer = JsFuture::from(peer_conn.create_offer()).await?;
    let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))?
        .as_string()
        .unwrap();
    console_log!("pc1 offer: {:?}", offer_sdp);

    let offer_obj = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
    offer_obj.set_sdp(&offer_sdp);
    let sld_promise = peer_conn.set_local_description(&offer_obj);
    JsFuture::from(sld_promise).await?;
    console_log!("pc1 state: {:?}", peer_conn.signaling_state());

    let offer_obj = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
    offer_obj.set_sdp(&offer_sdp);
    let srd_promise = p2.set_remote_description(&offer_obj);
    JsFuture::from(srd_promise).await?;
    console_log!("pc2 state: {:?}", p2.signaling_state());

    let answer = JsFuture::from(p2.create_answer()).await?;
    let answer_sdp = Reflect::get(&answer, &JsValue::from("sdp"))?
        .as_string()
        .unwrap();
    console_log!("pc2: answer: {:?}", answer_sdp);

    let answer_obj = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
    answer_obj.set_sdp(&answer_sdp);
    let sld_promise = p2.set_local_description(&answer_obj);
    JsFuture::from(sld_promise).await?;
    console_log!("pc2 state: {:?}", p2.signaling_state());

    let answer_obj = RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
    answer_obj.set_sdp(&answer_sdp);
    let srd_promise = peer_conn.set_remote_description(&answer_obj);
    JsFuture::from(srd_promise).await?;
    console_log!("pc1: state {:?}", peer_conn.signaling_state());

    Ok(())
}

#[wasm_bindgen]
pub fn respond_connect() {
    log("respond to connection");
}
