using NUnit.Framework;
namespace NeoDatis.Odb.Test.Transient_attributes
{
	public class VoWithTransientAttribute
	{
		private string name;

		[System.NonSerialized]
		private System.Collections.Generic.IList<string> keys;

		public VoWithTransientAttribute(string name)
		{
			this.name = name;
		}

		public virtual void AddKey(string key)
		{
			if (keys == null)
			{
				keys = new System.Collections.Generic.List<string>();
			}
			keys.Add(key);
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.Collections.Generic.IList<string> GetKeys()
		{
			return keys;
		}

		public virtual void SetKeys(System.Collections.Generic.IList<string> keys)
		{
			this.keys = keys;
		}
	}
}
