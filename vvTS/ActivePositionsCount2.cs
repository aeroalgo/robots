using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000DA RID: 218
	[HandlerCategory("vvTrade"), HandlerName("Количество активных позиций (2)")]
	public class ActivePositionsCount2 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000714 RID: 1812 RVA: 0x0001FA44 File Offset: 0x0001DC44
		public double Execute(ISecurity sec, int barNum)
		{
			IEnumerable<IPosition> activeForBar = sec.get_Positions().GetActiveForBar(barNum);
			return (double)activeForBar.Count<IPosition>();
		}
	}
}
