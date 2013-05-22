namespace NeoDatis.Odb.Test.Update.Nullobject
{
	/// <summary>User</summary>
	public class User
	{
		private NeoDatis.Odb.Test.Update.Nullobject.Profile profileId;

		private string login;

		private string name;

		private string password;

		private string email;

		private System.DateTime creationDate;

		private System.DateTime updateDate;

		private System.DateTime lastLogin;

		private bool status;

		private int rejectedLogin;

		private string sessionKey;

		private bool deleted;

		// S ou N
		public override string ToString()
		{
			return "[" + profileId + "]" + "[" + login + "][" + name + "][" + password + "]" 
				+ "[" + email + "][" + creationDate + "][" + lastLogin + "]" + "[" + status + "]["
				 + rejectedLogin + "]" + "][" + sessionKey + "][" + deleted + "]";
		}

		/// <returns>Returns the creationDate.</returns>
		public virtual System.DateTime GetCreationDate()
		{
			return creationDate;
		}

		/// <returns></returns>
		public virtual bool GetDeleted()
		{
			return deleted;
		}

		/// <returns>Returns the login.</returns>
		public virtual string GetLogin()
		{
			return login;
		}

		/// <param name="login">The login to set.</param>
		public virtual void SetLogin(string login)
		{
			this.login = login;
		}

		/// <returns>Returns the email.</returns>
		public virtual string GetEmail()
		{
			return email;
		}

		/// <returns>Returns the lastLogin.</returns>
		public virtual System.DateTime GetLastLogin()
		{
			return lastLogin;
		}

		/// <returns>Returns the name.</returns>
		public virtual string GetName()
		{
			return name;
		}

		/// <returns>Returns the password.</returns>
		public virtual string GetPassword()
		{
			return password;
		}

		/// <returns>Returns the profileId.</returns>
		public virtual NeoDatis.Odb.Test.Update.Nullobject.Profile GetProfileId()
		{
			return profileId;
		}

		/// <returns>Returns the rejectedLogin.</returns>
		public virtual int GetRejectedLogin()
		{
			return rejectedLogin;
		}

		/// <returns>Returns the sessionKey.</returns>
		public virtual string GetSessionKey()
		{
			return sessionKey;
		}

		/// <returns>Returns the status.</returns>
		public virtual bool GetStatus()
		{
			return status;
		}

		/// <returns>Returns the updateDate.</returns>
		public virtual System.DateTime GetUpdateDate()
		{
			return updateDate;
		}

		/// <param name="creationDate">The creationDate to set.</param>
		public virtual void SetCreationDate(System.DateTime creationDate)
		{
			this.creationDate = creationDate;
		}

		/// <param name="email">The email to set.</param>
		public virtual void SetEmail(string email)
		{
			this.email = email;
		}

		/// <param name="lastLogin">The lastLogin to set.</param>
		public virtual void SetLastLogin(System.DateTime lastLogin)
		{
			this.lastLogin = lastLogin;
		}

		/// <param name="name">The name to set.</param>
		public virtual void SetName(string name)
		{
			this.name = name;
		}

		/// <param name="password">The password to set.</param>
		public virtual void SetPassword(string password)
		{
			this.password = password;
		}

		/// <param name="profileId">The profileId to set.</param>
		public virtual void SetProfileId(NeoDatis.Odb.Test.Update.Nullobject.Profile profileId
			)
		{
			this.profileId = profileId;
		}

		/// <param name="rejectedLogin">The rejectedLogin to set.</param>
		public virtual void SetRejectedLogin(int rejectedLogin)
		{
			this.rejectedLogin = rejectedLogin;
		}

		/// <param name="sessionKey">The sessionKey to set.</param>
		public virtual void SetSessionKey(string sessionKey)
		{
			this.sessionKey = sessionKey;
		}

		/// <param name="status">The status to set.</param>
		public virtual void SetStatus(bool status)
		{
			this.status = status;
		}

		/// <param name="updateDate">The updateDate to set.</param>
		public virtual void SetUpdateDate(System.DateTime updateDate)
		{
			this.updateDate = updateDate;
		}

		/// <param name="deleted"></param>
		public virtual void SetDeleted(bool deleted)
		{
			this.deleted = deleted;
		}
	}
}
