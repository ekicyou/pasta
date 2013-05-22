namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>Profile</summary>
	public class Profile
	{
		private string name;

		public Profile(string name)
		{
			this.name = name;
		}

		public override string ToString()
		{
			return "[" + name + "]";
		}

		/// <returns>Returns the name.</returns>
		public virtual string GetName()
		{
			return name;
		}

		/// <param name="name">The name to set.</param>
		public virtual void SetName(string name)
		{
			this.name = name;
		}

		/// <summary>return boolean</summary>
		public override bool Equals(object obj)
		{
			if (obj == null)
			{
				return false;
			}
			else
			{
				if (!(obj is NeoDatis.Odb.Test.Update.Nullobject.Profile))
				{
					return false;
				}
				else
				{
					return name.Equals(((NeoDatis.Odb.Test.Update.Nullobject.Profile)obj).GetName());
				}
			}
		}
	}
}
