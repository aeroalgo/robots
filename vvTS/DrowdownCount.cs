using System;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000CA RID: 202
	[HandlerCategory("vvTrade"), HandlerName("Всего убыточных сделок")]
	public class DrowdownCount : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006E1 RID: 1761 RVA: 0x0001ED18 File Offset: 0x0001CF18
		public double Execute(ISecurity sec, int barNum)
		{
			return (double)(from pos in sec.get_Positions()
			orderby pos.get_ExitBarNum() descending
			where !pos.IsActiveForbar(barNum)
			select pos.Profit()).TakeWhile((double profit) => profit <= 0.0).Count<double>();
		}
	}
}
