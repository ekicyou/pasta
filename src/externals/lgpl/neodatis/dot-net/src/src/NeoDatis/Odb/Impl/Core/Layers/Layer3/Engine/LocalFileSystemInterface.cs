namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	public class LocalFileSystemInterface : NeoDatis.Odb.Core.Layers.Layer3.Engine.FileSystemInterface
	{
		protected NeoDatis.Odb.Core.Transaction.ISession session;

		public LocalFileSystemInterface(string name, NeoDatis.Odb.Core.Transaction.ISession
			 session, string fileName, bool canWrite, bool canLog, int bufferSize) : base(name
			, fileName, canWrite, canLog, bufferSize)
		{
			this.session = session;
		}

		public LocalFileSystemInterface(string name, NeoDatis.Odb.Core.Transaction.ISession
			 session, NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification parameters, bool canLog
			, int bufferSize) : base(name, parameters, canLog, bufferSize)
		{
			this.session = session;
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return session;
		}
	}
}
