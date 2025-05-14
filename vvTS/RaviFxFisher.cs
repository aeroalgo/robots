using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004B RID: 75
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Ravi FxFisher")]
	public class RaviFxFisher : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002AE RID: 686 RVA: 0x0000CD04 File Offset: 0x0000AF04
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("RaviFxFisher", new string[]
			{
				this.FastMAperiod.ToString(),
				this.SlowMAperiod.ToString(),
				sec.get_CacheName()
			}, () => RaviFxFisher.GenRaviFxFisher(sec, this.FastMAperiod, this.SlowMAperiod, this.Context));
		}

		// Token: 0x060002AD RID: 685 RVA: 0x0000CA64 File Offset: 0x0000AC64
		public static IList<double> GenRaviFxFisher(ISecurity sec, int _FastMAperiod, int _SlowMAperiod, IContext ctx)
		{
			IList<double> list = new List<double>(sec.get_Bars().Count);
			IList<double> typprice = ctx.GetData("TypicalPrice", new string[]
			{
				sec.get_CacheName()
			}, () => vvSeries.TypicalPrice(sec.get_Bars()));
			IList<double> data = ctx.GetData("lwma", new string[]
			{
				_FastMAperiod.ToString(),
				typprice.GetHashCode().ToString()
			}, () => LWMA.GenWMA(typprice, _FastMAperiod));
			IList<double> data2 = ctx.GetData("lwma", new string[]
			{
				_SlowMAperiod.ToString(),
				typprice.GetHashCode().ToString()
			}, () => LWMA.GenWMA(typprice, _SlowMAperiod));
			IList<double> data3 = ctx.GetData("atr", new string[]
			{
				_FastMAperiod.ToString(),
				sec.get_CacheName()
			}, () => ATR.GenATR(sec.get_Bars(), _FastMAperiod));
			IList<double> data4 = ctx.GetData("atr", new string[]
			{
				_SlowMAperiod.ToString(),
				sec.get_CacheName()
			}, () => ATR.GenATR(sec.get_Bars(), _SlowMAperiod));
			for (int i = 0; i < _SlowMAperiod; i++)
			{
				list.Add(0.0);
			}
			for (int j = _SlowMAperiod; j < sec.get_Bars().Count; j++)
			{
				double num = 100.0 * (data[j] - data2[j]) * data3[j] / data2[j] / data4[j];
				double item = (Math.Exp(2.0 * num) - 1.0) / (Math.Exp(2.0 * num) + 1.0);
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000E8 RID: 232
		public IContext Context
		{
			// Token: 0x060002AF RID: 687 RVA: 0x0000CD79 File Offset: 0x0000AF79
			get;
			// Token: 0x060002B0 RID: 688 RVA: 0x0000CD81 File Offset: 0x0000AF81
			set;
		}

		// Token: 0x170000E6 RID: 230
		[HandlerParameter(true, "4", Min = "1", Max = "50", Step = "1")]
		public int FastMAperiod
		{
			// Token: 0x060002A9 RID: 681 RVA: 0x0000C9CF File Offset: 0x0000ABCF
			get;
			// Token: 0x060002AA RID: 682 RVA: 0x0000C9D7 File Offset: 0x0000ABD7
			set;
		}

		// Token: 0x170000E7 RID: 231
		[HandlerParameter(true, "49", Min = "1", Max = "120", Step = "1")]
		public int SlowMAperiod
		{
			// Token: 0x060002AB RID: 683 RVA: 0x0000C9E0 File Offset: 0x0000ABE0
			get;
			// Token: 0x060002AC RID: 684 RVA: 0x0000C9E8 File Offset: 0x0000ABE8
			set;
		}
	}
}
