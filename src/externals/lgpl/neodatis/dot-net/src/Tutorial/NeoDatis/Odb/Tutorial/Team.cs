/*
NeoDatis ODB : Native Object Database (odb.info@neodatis.org)
Copyright (C) 2007 NeoDatis Inc. http://www.neodatis.org

This file is part of the db4o open source object database.

NeoDatis ODB is free software; you can redistribute it and/or
modify it under the terms of the GNU Lesser General Public
License as published by the Free Software Foundation; either
version 2.1 of the License, or (at your option) any later version.

NeoDatis ODB is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public
License along with this library; if not, write to the Free Software
Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
*/
using System;
using System.Collections;
using System.Text;
namespace Neodatis.Odb.Tutorial
{
    public class Team
    {
        private string name;
        private IList players;
        public Team(string name)
        {
            this.name = name;
            players = new ArrayList();
        }
        /**
         * @return the name
         */
        public string GetName()
        {
            return name;
        }
        /**
         * @param name the name to set
         */
        public void SetName(string name)
        {
            this.name = name;
        }
        /**
         * @return the players
         */
        public IList GetPlayers()
        {
            return players;
        }
        /**
         * @param players the players to set
         */
        public void SetPlayers(IList players)
        {
            this.players = players;
        }

        public void AddPlayer(Player player)
        {
            players.Add(player);
        }

        override public string ToString()
        {
            StringBuilder buffer = new StringBuilder();
            buffer.Append("Team ").Append(name).Append(" ").Append(players);
            return buffer.ToString();
        }

    }
}


