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
using System.Text;
namespace Neodatis.Odb.Tutorial
{
    public class Game
    {
        private DateTime when;
        private Sport sport;
        private Team team1;
        private Team team2;
        private string result;


        public Game(DateTime when, Sport sport, Team team1, Team team2)
        {
            this.when = when;
            this.sport = sport;
            this.team1 = team1;
            this.team2 = team2;
        }
        public String GetResult()
        {
            return result;
        }
        public void SetResult(String result)
        {
            this.result = result;
        }
        public Sport GetSport()
        {
            return sport;
        }
        public void SetSport(Sport sport)
        {
            this.sport = sport;
        }
        public Team GetTeam1()
        {
            return team1;
        }
        public void SetTeam1(Team team1)
        {
            this.team1 = team1;
        }
        public Team GetTeam2()
        {
            return team2;
        }
        public void SetTeam2(Team team2)
        {
            this.team2 = team2;
        }
        override public string ToString()
        {
            StringBuilder buffer = new StringBuilder();
            buffer.Append(when).Append(" : Game of ").Append(sport).Append(" between ").Append(team1.GetName()).Append(" and ").Append(team2.GetName());
            return buffer.ToString();
        }
    }
}



