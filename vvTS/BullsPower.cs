using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000015 RID: 21
	[HandlerCategory("vvIndicators"), HandlerName("Bulls Power")]
	public class BullsPower : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000AB RID: 171 RVA: 0x00003FC8 File Offset: 0x000021C8
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.Period.ToString(),
				sec.get_CacheName()
			}, () => EMA.GenEMA(sec.get_ClosePrices(), this.Period));
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
					item = highPrices[i] - data[i];
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x17000037 RID: 55
		public IContext Context
		{
			// Token: 0x060000AC RID: 172 RVA: 0x000040AF File Offset: 0x000022AF
			get;
			// Token: 0x060000AD RID: 173 RVA: 0x000040B7 File Offset: 0x000022B7
			set;
		}

		// Token: 0x17000036 RID: 54
		[HandlerParameter(true, "13", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x060000A9 RID: 169 RVA: 0x00003F90 File Offset: 0x00002190
			get;
			// Token: 0x060000AA RID: 170 RVA: 0x00003F98 File Offset: 0x00002198
			set;
		}
	}
}
