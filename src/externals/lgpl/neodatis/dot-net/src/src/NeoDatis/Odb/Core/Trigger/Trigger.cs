namespace NeoDatis.Odb.Core.Trigger
{
	/// <summary>A simple base class for all triggers.</summary>
	/// <remarks>A simple base class for all triggers.</remarks>
	/// <author>olivier</author>
	public class Trigger
	{
		private NeoDatis.Odb.ODB odb;

		public virtual void SetOdb(NeoDatis.Odb.ODB odb)
		{
			this.odb = odb;
		}

		public virtual NeoDatis.Odb.ODB GetOdb()
		{
			return odb;
		}
	}
}
