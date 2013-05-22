using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Login
{
	public class Profile
	{
		private string name;

		private System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Function> functions;

		public Profile()
		{
		}

		public Profile(string name) : base()
		{
			this.name = name;
		}

		public Profile(string name, System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Function
			> functions) : base()
		{
			this.functions = functions;
			this.name = name;
		}

		public Profile(string name, NeoDatis.Odb.Test.VO.Login.Function function) : base(
			)
		{
			this.functions = new System.Collections.Generic.List<NeoDatis.Odb.Test.VO.Login.Function
				>();
			this.functions.Add(function);
			this.name = name;
		}

		public virtual void AddFunction(NeoDatis.Odb.Test.VO.Login.Function function)
		{
			if (functions == null)
			{
				functions = new System.Collections.Generic.List<Function>();
			}
			functions.Add(function);
		}

		public virtual System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Function
			> GetFunctions()
		{
			return functions;
		}

		public virtual void SetFunctions(System.Collections.Generic.IList<NeoDatis.Odb.Test.VO.Login.Function
			> functions)
		{
			this.functions = functions;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public override string ToString()
		{
			return name + " - " + (functions != null ? functions.ToString() : "null");
		}

		public virtual bool Equals2(object obj)
		{
			if (obj == null || obj.GetType() != typeof(NeoDatis.Odb.Test.VO.Login.Profile))
			{
				return false;
			}
			NeoDatis.Odb.Test.VO.Login.Profile p = (NeoDatis.Odb.Test.VO.Login.Profile)obj;
			if (name == null && p.name != null)
			{
				return false;
			}
			return (name == null && p.name == null) || (name.Equals(p.name));
		}
	}
}
