using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FF RID: 255
	[HandlerCategory("vvTrade"), HandlerName("Всего баров")]
	public class TotalBars : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000764 RID: 1892 RVA: 0x000209B2 File Offset: 0x0001EBB2
		public double Execute(ISecurity sec, int barNum)
		{
			return (double)sec.get_Bars().Count;
		}
	}
}
