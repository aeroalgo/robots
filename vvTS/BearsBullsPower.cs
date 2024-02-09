using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000013 RID: 19
	[HandlerCategory("vvIndicators"), HandlerName("Bears Bulls Power")]
	public class BearsBullsPower : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600009F RID: 159 RVA: 0x00003DDC File Offset: 0x00001FDC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("BearsBullsPower", new string[]
			{
				this.Period.ToString(),
				sec.get_CacheName()
			}, () => BearsBullsPower.GenBearsBullsPower(sec, this.Period, this.Context));
		}

		// Token: 0x0600009E RID: 158 RVA: 0x00003CD8 File Offset: 0x00001ED8
		public static IList<double> GenBearsBullsPower(ISecurity sec, int period, IContext ctx)
		{
			int count = sec.get_Bars().Count;
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> Close = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> list = new double[count];
			IList<double> data = ctx.GetData("ema", new string[]
			{
				period.ToString(),
				sec.get_CacheName()
			}, () => EMA.GenEMA(Close, period));
			for (int i = 0; i < count; i++)
			{
				double num = highPrices[i] - data[i];
				double num2 = lowPrices[i] - data[i];
				list[i] = (num2 + num) / 2.0;
			}
			return list;
		}

		// Token: 0x17000033 RID: 51
		public IContext Context
		{
			// Token: 0x060000A0 RID: 160 RVA: 0x00003E40 File Offset: 0x00002040
			get;
			// Token: 0x060000A1 RID: 161 RVA: 0x00003E48 File Offset: 0x00002048
			set;
		}

		// Token: 0x17000032 RID: 50
		[HandlerParameter(true, "13", Min = "9", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x0600009C RID: 156 RVA: 0x00003CAB File Offset: 0x00001EAB
			get;
			// Token: 0x0600009D RID: 157 RVA: 0x00003CB3 File Offset: 0x00001EB3
			set;
		}
	}
}
