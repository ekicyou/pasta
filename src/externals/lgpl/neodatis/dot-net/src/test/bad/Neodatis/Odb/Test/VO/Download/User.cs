namespace NeoDatis.Odb.Test.VO.Download
{
	public class User
	{
		private string name;

		private string email;

		private string runtimeVersion;

		private string country;

		private string city;

		private int nbDownloads;

		private System.DateTime lastDownload;

		public User() : base()
		{
		}

		public virtual string GetCity()
		{
			return city;
		}

		public virtual void SetCity(string city)
		{
			this.city = city;
		}

		public virtual string GetCountry()
		{
			return country;
		}

		public virtual void SetCountry(string country)
		{
			this.country = country;
		}

		public virtual string GetEmail()
		{
			return email;
		}

		public virtual void SetEmail(string email)
		{
			this.email = email;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual string GetRuntimeVersion()
		{
			return runtimeVersion;
		}

		public virtual void SetRuntimeVersion(string runtimeVersion)
		{
			this.runtimeVersion = runtimeVersion;
		}

		public virtual int GetNbDownloads()
		{
			return nbDownloads;
		}

		public virtual void SetNbDownloads(int nbDownloads)
		{
			this.nbDownloads = nbDownloads;
		}

		public virtual System.DateTime GetLastDownload()
		{
			return lastDownload;
		}

		public virtual void SetLastDownload(System.DateTime lastDownload)
		{
			this.lastDownload = lastDownload;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("name=").Append(name).Append(" - email=").Append(email);
			buffer.Append("lastDownload").Append(lastDownload).Append(" - nb downloads=").Append
				(nbDownloads);
			return buffer.ToString();
		}
	}
}
