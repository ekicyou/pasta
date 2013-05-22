namespace NeoDatis.Odb.Core.Server.Layers.Layer3
{
	/// <summary>Database Parameters for local database access</summary>
	/// <author>osmadja</author>
	public class ServerFileParameter : NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
	{
		private string baseName;

		public ServerFileParameter(string baseName, string fileName, bool write, string userName
			, string password) : base(fileName, write, userName, password)
		{
			this.baseName = baseName;
		}

		public virtual string GetBaseName()
		{
			return baseName;
		}

		public virtual void SetBaseName(string baseName)
		{
			this.baseName = baseName;
		}

		public override string GetIdentification()
		{
			return baseName;
		}

		public override bool IsLocal()
		{
			return false;
		}
	}
}
