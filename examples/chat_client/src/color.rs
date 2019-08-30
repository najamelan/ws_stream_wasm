#![ allow( dead_code )]

use crate::import::*;



pub struct Color
{
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}


impl Color
{
	pub fn new( r: u8, g: u8, b: u8, a: u8 ) -> Self
	{
		Self{ r, g, b, a }
	}

	pub fn random() -> Self
	{
		Self
		{
			r: ( Math::random() * 255_f64 ) as u8,
			g: ( Math::random() * 255_f64 ) as u8,
			b: ( Math::random() * 255_f64 ) as u8,
			a: ( Math::random() * 255_f64 ) as u8,
		}
	}


	// If this color is darker than half luminosity, it will be inverted
	//
	pub fn light( self ) -> Self
	{
		if self.is_dark() { self.invert() }
		else { self }
	}


	// If this color is lighter than half luminosity, it will be inverted
	//
	pub fn dark( self ) -> Self
	{
		if self.is_light() { self.invert() }
		else { self }
	}


	/// Invert color.
	//
	pub fn invert( mut self ) -> Self
	{
		self.r = 255 - self.r;
		self.g = 255 - self.g;
		self.b = 255 - self.b;
		self.a = 255 - self.a;

		self
	}


	// True if this color is lighter than half luminosity.
	//
	pub fn is_light( &self ) -> bool
	{
		self.r as u16 + self.g as u16 + self.b as u16 > 378 // 128 * 3
	}


	/// True if this color is darker than half luminosity.
	//
	pub fn is_dark( &self ) -> bool
	{
		!self.is_light()
	}


	// output a css string format: "#rrggbb"
	//
	pub fn to_css( &self ) -> String
	{
		format!( "#{:02x}{:02x}{:02x}", self.r, self.g, self.b )
	}


	// output a css string format: "rgba( rrr, ggg, bbb, aaa )"
	//
	pub fn to_cssa( &self ) -> String
	{
		format!( "rgba({},{},{},{})", self.r, self.g, self.b, self.a )
	}
}


#[ cfg(test) ]
mod tests
{
	use super::*;

	#[test]
	//
	fn padding()
	{
		let c = Color::new( 1, 1, 1, 1 );

		assert_eq!( "#010101", c.to_css() );
	}
}


