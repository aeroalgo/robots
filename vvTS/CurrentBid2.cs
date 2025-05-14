using System;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Realtime;

namespace vvTSLtools
{
	// Token: 0x020000C3 RID: 195
	[HandlerCategory("vvTrade"), HandlerName("Текущий Bid 2")]
	public class CurrentBid2 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006D3 RID: 1747 RVA: 0x0001E9B8 File Offset: 0x0001CBB8
		public double Execute(ISecurity sec)
		{
			ISecurityRt securityRt = sec as ISecurityRt;
			return (securityRt == null) ? 0.0 : (securityRt.get_FinInfo().get_Bid().HasValue ? securityRt.get_FinInfo().get_Bid().Value : 0.0);
		}
	}
}
