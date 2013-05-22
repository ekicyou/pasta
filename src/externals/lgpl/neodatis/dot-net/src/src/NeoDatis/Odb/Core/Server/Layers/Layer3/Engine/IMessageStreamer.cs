namespace NeoDatis.Odb.Core.Server.Layers.Layer3.Engine
{
	/// <author>olivier</author>
	public interface IMessageStreamer
	{
		void Close();

		void Write(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message message);

		NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message Read();
	}
}
