namespace NeoDatis.Btree.Tool
{
	/// <summary>an utility to display a btree</summary>
	/// <author>osmadja</author>
	public class BTreeDisplay
	{
		private System.Text.StringBuilder[] lines;

		private System.Text.StringBuilder result;

		public BTreeDisplay()
		{
		}

		public virtual System.Text.StringBuilder Build(NeoDatis.Btree.IBTree btree, bool 
			withIds)
		{
			lines = new System.Text.StringBuilder[btree.GetHeight()];
			for (int i = 0; i < btree.GetHeight(); i++)
			{
				lines[i] = new System.Text.StringBuilder();
			}
			BuildDisplay(btree.GetRoot(), 0, 0, "0", withIds);
			BuildRepresentation();
			return result;
		}

		public virtual System.Text.StringBuilder Build(NeoDatis.Btree.IBTreeNode node, int
			 height, bool withIds)
		{
			lines = new System.Text.StringBuilder[height];
			for (int i = 0; i < height; i++)
			{
				lines[i] = new System.Text.StringBuilder();
			}
			BuildDisplay(node, 0, 0, "0", withIds);
			BuildRepresentation();
			return result;
		}

		private void BuildRepresentation()
		{
			int maxLineSize = lines[lines.Length - 1].Length;
			result = new System.Text.StringBuilder();
			for (int i = 0; i < lines.Length; i++)
			{
				result.Append(Format(lines[i], i, maxLineSize)).Append("\n");
			}
		}

		public virtual System.Text.StringBuilder GetResult()
		{
			return result;
		}

		private System.Text.StringBuilder Format(System.Text.StringBuilder line, int height
			, int maxLineSize)
		{
			int diff = maxLineSize - line.Length;
			System.Text.StringBuilder lineResult = new System.Text.StringBuilder();
			lineResult.Append("h=").Append(height + 1).Append(":");
			lineResult.Append(Fill(diff / 2, ' '));
			lineResult.Append(line);
			lineResult.Append(Fill(diff / 2, ' '));
			return lineResult;
		}

		private System.Text.StringBuilder Fill(int size, char c)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < size; i++)
			{
				buffer.Append(c);
			}
			return buffer;
		}

		private void BuildDisplay(NeoDatis.Btree.IBTreeNode node, int currentHeight, int 
			childIndex, object parentId, bool withIds)
		{
			if (currentHeight > lines.Length - 1)
			{
				return;
			}
			// get string buffer of this line
			System.Text.StringBuilder line = lines[currentHeight];
			if (withIds)
			{
				line.Append(node.GetId()).Append(":[");
			}
			else
			{
				line.Append("[");
			}
			for (int i = 0; i < node.GetNbKeys(); i++)
			{
				if (i > 0)
				{
					line.Append(" , ");
				}
				NeoDatis.Btree.IKeyAndValue kav = node.GetKeyAndValueAt(i);
				line.Append(kav.GetKey());
			}
			if (withIds)
			{
				line.Append("]:").Append(node.GetParentId()).Append("/").Append(parentId).Append(
					"    ");
			}
			else
			{
				line.Append("]  ");
			}
			for (int i = 0; i < node.GetNbChildren(); i++)
			{
				NeoDatis.Btree.IBTreeNode child = node.GetChildAt(i, false);
				if (child != null)
				{
					BuildDisplay(child, currentHeight + 1, i, node.GetId(), withIds);
				}
				else
				{
					lines[currentHeight + 1].Append("[Child " + (i + 1) + " null!] ");
				}
			}
		}
	}
}
