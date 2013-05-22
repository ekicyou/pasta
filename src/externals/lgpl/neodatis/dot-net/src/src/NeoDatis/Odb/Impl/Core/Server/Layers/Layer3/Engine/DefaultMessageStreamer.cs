	namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
	{
	
		using System.IO;
		using NeoDatis.Odb.Core.Server.Layers.Layer3.Engine;
		
		public class DefaultMessageStreamer
		{
		private string host;
		private int port;
		private string name;
		public DefaultMessageStreamer(string host, int port,string name){
			this.host = host;
			this.port = port;
			this.name = name;
			InitSocket();
		}
		
		public DefaultMessageStreamer(Stream in2, Stream out2,
				BinaryReader ois2, BinaryWriter oos2) {
		
		}
	
	
		private void InitSocket() {
			
		}
	
		public void closeSocket() {
			
	
		}
	
	
		public void Write(Message message) {
					//Serialize(stream, message);
	
		}
	
		public Message Read() {
			try
			{
				return null;//(Message) Deserialize(stream);
			}catch (System.Exception e)
			{
			throw new System.IO.IOException(e.Message);
			}
		}
		
		/*******************************/
		/// <summary>
		/// Writes an object to the specified Stream
		/// </summary>
		/// <param name="stream">The target Stream</param>
		/// <param name="objectToSend">The object to be sent</param>
		public static void Serialize(System.IO.Stream stream, System.Object objectToSend)
		{
			System.Runtime.Serialization.Formatters.Binary.BinaryFormatter formatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
			formatter.Serialize(stream, objectToSend);
		}
	
		/// <summary>
		/// Writes an object to the specified BinaryWriter
		/// </summary>
		/// <param name="stream">The target BinaryWriter</param>
		/// <param name="objectToSend">The object to be sent</param>
		public static void Serialize(System.IO.BinaryWriter binaryWriter, System.Object objectToSend)
		{
			System.Runtime.Serialization.Formatters.Binary.BinaryFormatter formatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
			formatter.Serialize(binaryWriter.BaseStream, objectToSend);
		}
	
		/*******************************/
		/// <summary>
		/// Deserializes an object, or an entire graph of connected objects, and returns the object intance
		/// </summary>
		/// <param name="binaryReader">Reader instance used to read the object</param>
		/// <returns>The object instance</returns>
		public static System.Object Deserialize(System.IO.BinaryReader binaryReader)
		{
			System.Runtime.Serialization.Formatters.Binary.BinaryFormatter formatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
			return formatter.Deserialize(binaryReader.BaseStream);
		}
	
		
		
		}	
	}