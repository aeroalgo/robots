using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000014 RID: 20
	[HandlerCategory("vvIndicators"), HandlerName("Bears Power")]
	public class BearsPower : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000A5 RID: 165 RVA: 0x00003E90 File Offset: 0x00002090
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.Period.ToString(),
				sec.get_CacheName()
			}, () => Series.EMA(sec.get_ClosePrices(), this.Period));
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
					item = lowPrices[i] - data[i];
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x17000035 RID: 53
		public IContext Context
		{
			// Token: 0x060000A6 RID: 166 RVA: 0x00003F77 File Offset: 0x00002177
			get;
			// Token: 0x060000A7 RID: 167 RVA: 0x00003F7F File Offset: 0x0000217F
			set;
		}

		// Token: 0x17000034 RID: 52
		[HandlerParameter(true, "13", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x060000A3 RID: 163 RVA: 0x00003E59 File Offset: 0x00002059
			get;
			// Token: 0x060000A4 RID: 164 RVA: 0x00003E61 File Offset: 0x00002061
			set;
		}
	}
}
