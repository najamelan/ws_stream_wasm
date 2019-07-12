use crate::import::*;


/// Turn a stream/sink of binary messages into a stream of bytes (AsyncRead/AsyncWrite).
//
#[ derive( Debug, Clone ) ]
//
pub struct ChunkStream
{
	queue: VecDeque<Uint8Array>,

	// The u32 is the pointer into the first element on the queue. Since the buffer fed
	// to read is not necessarily of the same size of the messages we receive, this
	// pointer allows to keep track of the data we have already read.
	//
	already_read: u32,
}



impl ChunkStream
{
	/// Create a new ChunkStream
	//
	pub fn new() -> Self
	{
		Self { queue: VecDeque::new(), already_read: 0 }
	}


	/// Whether the queue is empty (no data left to read)
	//
	#[ inline( always ) ]
	//
	pub fn is_empty( &self ) -> bool
	{
		self.queue.is_empty()
	}


	/// Add a new message to the queue
	//
	#[ inline( always ) ]
	//
	pub fn push( &mut self, msg: Uint8Array )
	{
		self.queue.push_back( msg );
	}

	/// Try to read some bytes.
	/// FIXME? Do not call this method if the queue is empty!
	//
	// We have a queue of Uint8Arrays with incoming data, and a buf &mut [u8] to copy it to. Ideally we would like to consider
	// the queue as one long buffer, but it's not, so we keep track of:
	// - already_read: the offset in the first array on the queue until where we have already read
	// - as soon as we completely read an array, we pop it from the queue and set already_read to 0
	// - if we can't fit an entire array in the buffer, we set already_read to the correct offset and return
	//
	//
	// this method returns the number of bytes that where copied.
	//
	#[ allow( clippy::needless_return )]
	//
	pub fn read( &mut self, mut copied: u32, buf: &mut [u8] ) -> u32
	{

		let space   = buf.len() as u32                           ;
		let data    = self.queue[0].length() - self.already_read ;
		let to_copy = min( data, space )                         ;
		let end     = self.already_read + to_copy                ;


		// Buffer is exactly the right size, just copy it over.
		//
		if data == space
		{
			if   self.already_read == 0 { self.queue[0]                                   .copy_to( &mut buf[ ..(to_copy as usize) ] ); }
			else                        { self.queue[0].subarray( self.already_read, end ).copy_to( &mut buf[ ..(to_copy as usize) ] ); }

			self.pop();

			return copied + to_copy;
		}


		// The first message has less data than the buffer can hold.
		// We will copy the first message entirely and then see if there are any more messges to copy.
		//
		else if data < space
		{
			if   self.already_read == 0 { self.queue[0]                                .copy_to( &mut buf[ ..(to_copy as usize) ] ); }
			else                        { self.queue[0].slice( self.already_read, end ).copy_to( &mut buf[ ..(to_copy as usize) ] ); }

			self.pop();
			copied += to_copy;

			// Only recurse if there are more arrays waiting to be copied
			//
			if self.queue.is_empty() { return copied }

			else
			{
				return self.read( copied, &mut buf[ (to_copy as usize).. ] )
			}
		}


		// The first message has more data than the buffer can hold.
		//
		else // if data > space
		{
			self.queue[0].slice( self.already_read, end ).copy_to( &mut buf[ ..(to_copy as usize) ] );

			self.already_read = end;

			return copied + to_copy;
		}
	}


	#[ inline( always ) ]
	//
	fn pop( &mut self )
	{
		self.queue.pop_front();
		self.already_read = 0;
	}
}
