using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DB RID: 219
	[HandlerCategory("vvTrade"), HandlerName("Количество активных позиций Long")]
	public class ActiveLongsCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000716 RID: 1814 RVA: 0x0001FA70 File Offset: 0x0001DC70
		public double Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> activeForBar = sec.get_Positions().GetActiveForBar(barNum);
			if (activeForBar.Count<IPosition>() < 1)
			{
				return 0.0;
			}
			double num = 0.0;
			foreach (IPosition current in activeForBar)
			{
				if (current.get_IsLong())
				{
					num += 1.0;
				}
			}
			return num;
		}
	}
}
