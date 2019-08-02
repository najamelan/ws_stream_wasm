use crate :: { import::*, color::*, document };


pub struct User
{
	sid   : usize                ,
	nick  : String               ,
	color : Color                ,
	p     : HtmlParagraphElement ,
	indom : bool                 ,
}


impl User
{
	pub fn new( sid: usize, nick: String ) -> Self
	{
		Self
		{
			nick  ,
			sid   ,
			color: Color::random().light(),
			p    : document().create_element( "p" ).expect( "create user p" ).unchecked_into(),
			indom: false,
		}
	}


	pub fn render( &mut self, parent: &HtmlElement )
	{
		self.p.set_inner_text( &self.nick );

		self.p.style().set_property( "color", &self.color.to_css() ).expect_throw( "set color" );
		self.p.set_id( &format!( "user_{}", &self.sid ) );

		parent.append_child( &self.p ).expect_throw( "append user to div" );

		self.indom = true;
	}


	pub fn change_nick( &mut self, new: String )
	{
		self.nick = new;

		if self.indom { self.render( &self.p.parent_node().expect_throw( "get user parent node" ).unchecked_into() ) }
	}
}



impl Drop for User
{
	fn drop( &mut self )
	{
		debug!( "removing user from dom" );
		self.p.remove();
	}
}



pub struct UserList
{
	users: HashMap<usize, User>,
	div  : HtmlDivElement      ,
	indom: bool                ,
}



impl UserList
{
	pub fn new() -> Self
	{
		Self
		{
			users: HashMap::new() ,
			div  : document().create_element( "div" ).expect( "create userlist div" ).unchecked_into() ,
			indom: false,
		}
	}


	pub fn insert( &mut self, sid: usize, nick: String )
	{
		let mut render = false;

		let user = self.users.entry( sid )

			// TODO: Get rid of clone
			// existing users know if they are in the dom, so we don't call render on them.
			//
			.and_modify( |usr| usr.change_nick( nick.clone() ) )

			.or_insert_with ( ||
			{
				render = true;
				User::new( sid, nick )
			})

		;

		if render { user.render( &self.div ); }
	}


	pub fn remove( &mut self, sid: usize )
	{
		self.users.remove( &sid );
	}



	pub fn render( &mut self, parent: &HtmlElement )
	{
		for ref mut user in self.users.values_mut()
		{
			user.render( &self.div );
		}

		parent.append_child( &self.div ).expect_throw( "add udiv to dom" );

		self.indom = true;
	}
}


impl Drop for UserList
{
	fn drop( &mut self )
	{
		// remove self from Dom
		//
		self.div.remove();
		//
		// Delete children
		//

	}
}
