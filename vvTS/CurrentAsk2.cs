using System;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C5 RID: 197
	[HandlerCategory("vvTrade"), HandlerName("Текущий Ask 2")]
	public class CurrentAsk2 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006D7 RID: 1751 RVA: 0x0001EA9C File Offset: 0x0001CC9C
		public double Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			return (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_Ask().HasValue ? securityRt.get_FinInfo().get_Ask().Value : 0.0);
		}
	}
}
