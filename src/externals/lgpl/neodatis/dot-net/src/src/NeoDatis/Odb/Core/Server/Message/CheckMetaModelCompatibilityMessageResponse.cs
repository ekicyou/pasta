namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class CheckMetaModelCompatibilityMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult result;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel updatedMetaModel;

		public CheckMetaModelCompatibilityMessageResponse(string baseId, string sessionId
			, NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult result, NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
			 metaModel) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.CheckMetaModelCompatibility
			, baseId, sessionId)
		{
			this.result = result;
			this.updatedMetaModel = metaModel;
		}

		public CheckMetaModelCompatibilityMessageResponse(string baseId, string sessionId
			, string error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.CheckMetaModelCompatibility
			, baseId, sessionId)
		{
			SetError(error);
		}

		public override string ToString()
		{
			return "CheckMetaModelCompatibility";
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult GetResult
			()
		{
			return result;
		}

		public virtual void SetResult(NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult
			 result)
		{
			this.result = result;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetUpdatedMetaModel
			()
		{
			return updatedMetaModel;
		}

		public virtual void SetUpdatedMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
			 updatedMetaModel)
		{
			this.updatedMetaModel = updatedMetaModel;
		}
	}
}
