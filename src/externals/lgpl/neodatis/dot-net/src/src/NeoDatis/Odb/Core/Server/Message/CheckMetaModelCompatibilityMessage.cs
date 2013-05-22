namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class CheckMetaModelCompatibilityMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> currentCIs;

		public CheckMetaModelCompatibilityMessage(string baseId, string sessionId, System.Collections.Generic.IDictionary
			<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo> currentCIs) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.CheckMetaModelCompatibility, baseId, sessionId)
		{
			this.currentCIs = currentCIs;
		}

		public override string ToString()
		{
			return "CheckMetaModelCompatibility";
		}

		public virtual System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> GetCurrentCIs()
		{
			return currentCIs;
		}

		public virtual void SetCurrentCIs(System.Collections.Generic.IDictionary<string, 
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo> currentCIs)
		{
			this.currentCIs = currentCIs;
		}
	}
}
