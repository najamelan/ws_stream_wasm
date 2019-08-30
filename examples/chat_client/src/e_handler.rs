use crate::import::*;


pub struct EHandler
{
	receiver: UnboundedReceiver<Event>,

	// Automatically removed from the DOM on drop!
	//
	_listener: EventListener,
}


impl EHandler
{
	pub fn new( target: &EventTarget, event: &'static str, passive: bool ) -> Self
	{
		// debug!( "set event handler" );

		let (sender, receiver) = unbounded();
		let options = match passive
		{
			false => EventListenerOptions::enable_prevent_default(),
			true  => EventListenerOptions::default(),
		};

		// Attach an event listener
		//
		let _listener = EventListener::new_with_options( &target, event, options, move |event|
		{
			sender.unbounded_send(event.clone()).unwrap_throw();
		});

		Self
		{
			receiver,
			_listener,
		}
	}
}



impl Stream for EHandler
{
	type Item = Event;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>
	{
		Pin::new( &mut self.receiver ).poll_next(cx)
	}
}

