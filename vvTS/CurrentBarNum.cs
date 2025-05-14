using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FE RID: 254
	[HandlerCategory("vvTrade"), HandlerName("Номер текущего бара")]
	public class CurrentBarNum : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000762 RID: 1890 RVA: 0x000209A6 File Offset: 0x0001EBA6
		public double Execute(ISecurity sec, int barNum)
		{
			return (double)barNum;
		}
	}
}
