using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000048 RID: 72
	[HandlerCategory("vvIndicators"), HandlerName("Price and Volume Trend")]
	public class PVT : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600028F RID: 655 RVA: 0x0000C244 File Offset: 0x0000A444
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> volumes = sec.get_Volumes();
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double item;
				if (i < this.Period)
				{
					item = 0.0;
				}
				else
				{
					item = (closePrices[i] - closePrices[i - this.Period]) / closePrices[i - this.Period] * volumes[i] + list[i - 1];
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000DD RID: 221
		public IContext Context
		{
			// Token: 0x06000290 RID: 656 RVA: 0x0000C2E5 File Offset: 0x0000A4E5
			get;
			// Token: 0x06000291 RID: 657 RVA: 0x0000C2ED File Offset: 0x0000A4ED
			set;
		}

		// Token: 0x170000DC RID: 220
		[HandlerParameter(true, "1", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x0600028D RID: 653 RVA: 0x0000C233 File Offset: 0x0000A433
			get;
			// Token: 0x0600028E RID: 654 RVA: 0x0000C23B File Offset: 0x0000A43B
			set;
		}
	}
}
