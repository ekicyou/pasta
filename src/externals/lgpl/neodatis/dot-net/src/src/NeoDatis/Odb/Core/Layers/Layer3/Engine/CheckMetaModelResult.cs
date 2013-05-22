namespace NeoDatis.Odb.Core.Layers.Layer3.Engine
{
	/// <author>olivier</author>
	[System.Serializable]
	public class CheckMetaModelResult
	{
		private bool modelHasBeenUpdated;

		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoCompareResult
			> results;

		public CheckMetaModelResult()
		{
			this.modelHasBeenUpdated = false;
			this.results = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoCompareResult
				>();
		}

		public virtual bool IsModelHasBeenUpdated()
		{
			return modelHasBeenUpdated;
		}

		public virtual void SetModelHasBeenUpdated(bool modelHasBeenUpdated)
		{
			this.modelHasBeenUpdated = modelHasBeenUpdated;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoCompareResult
			> GetResults()
		{
			return results;
		}

		public virtual void SetResults(NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoCompareResult
			> results)
		{
			this.results = results;
		}

		public virtual void Add(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoCompareResult
			 result)
		{
			this.results.Add(result);
		}

		public virtual int Size()
		{
			return this.results.Count;
		}
	}
}
