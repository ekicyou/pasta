namespace NeoDatis.Odb.Core.Layers.Layer2.Instance
{
	public interface IInstanceBuilder
	{
		/// <summary>Builds a Non Native Object instance TODO Perf checks the IFs Builds a non native object using The object info
		/// 	</summary>
		/// <param name="objectInfo"></param>
		/// <returns>The instance</returns>
		/// <></>
		object BuildOneInstance(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo 
			objectInfo);

		/// <summary>Returns the session id of this instance builder (odb database identifier)
		/// 	</summary>
		/// <returns></returns>
		string GetSessionId();

		/// <summary>To specify if instance builder is part of local StorageEngine.</summary>
		/// <remarks>
		/// To specify if instance builder is part of local StorageEngine. In server mode, for instance, when called on
		/// the server, it will return false
		/// </remarks>
		/// <returns></returns>
		bool IsLocal();
	}
}
