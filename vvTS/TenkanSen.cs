using System;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200007C RID: 124
	[HandlerCategory("vvIchimoku"), HandlerName("TenkanSen")]
	public class TenkanSen : BaseSen, IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x17000182 RID: 386
		[HandlerParameter(true, "9", Min = "3", Max = "25", Step = "1")]
		public override int Period
		{
			// Token: 0x0600046D RID: 1133 RVA: 0x000174CC File Offset: 0x000156CC
			get;
			// Token: 0x0600046E RID: 1134 RVA: 0x000174D4 File Offset: 0x000156D4
			set;
		}
	}
}
