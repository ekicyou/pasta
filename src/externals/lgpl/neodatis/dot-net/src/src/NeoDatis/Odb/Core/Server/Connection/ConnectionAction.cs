namespace NeoDatis.Odb.Core.Server.Connection
{
	/// <summary>A simple class with some constants to describe what a connection is doing
	/// 	</summary>
	/// <author>osmadja</author>
	public class ConnectionAction
	{
		public const int ActionNoAction = -1;

		public static readonly string ActionNoActionLabel = "-";

		public const int ActionConnect = 0;

		public static readonly string ActionConnectLabel = "connect";

		public const int ActionInsert = 1;

		public static readonly string ActionInsertLabel = "insert";

		public const int ActionUpdate = 2;

		public static readonly string ActionUpdateLabel = "update";

		public const int ActionDelete = 3;

		public static readonly string ActionDeleteLabel = "delete";

		public const int ActionSelect = 4;

		public static readonly string ActionSelectLabel = "select";

		public const int ActionCommit = 5;

		public static readonly string ActionCommitLabel = "commit";

		public const int ActionClose = 6;

		public static readonly string ActionCloseLabel = "close";

		public const int ActionRollback = 7;

		public static readonly string ActionRollbackLabel = "rollback";

		protected static readonly string[] ActionLabels = new string[] { ActionConnectLabel
			, ActionInsertLabel, ActionUpdateLabel, ActionDeleteLabel, ActionSelectLabel, ActionCommitLabel
			, ActionCloseLabel, ActionRollbackLabel };

		public static int GetNumberOfActions()
		{
			return ActionLabels.Length;
		}

		public static string GetActionLabel(int action)
		{
			if (action == ActionNoAction)
			{
				return ActionNoActionLabel;
			}
			return ActionLabels[action];
		}
	}
}
