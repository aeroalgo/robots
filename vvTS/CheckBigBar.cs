using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000112 RID: 274
	[HandlerCategory("vvTrade"), HandlerName("Бар больше предыдущего\nбара в Х.Х раз")]
	public class CheckBigBar : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060007AA RID: 1962 RVA: 0x000219E4 File Offset: 0x0001FBE4
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> openPrices = src.get_OpenPrices();
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> list = new double[count];
			for (int i = 1; i < count; i++)
			{
				list[i] = 0.0;
				double num = Math.Abs(openPrices[i] - closePrices[i]);
				double num2 = Math.Abs(openPrices[i - 1] - closePrices[i - 1]);
				if (num2 * this.Coef < num)
				{
					list[i] = 1.0;
				}
			}
			return list;
		}

		// Token: 0x1700026D RID: 621
		[HandlerParameter(true, "2.0", Min = "1.5", Max = "3", Step = "0.1")]
		public double Coef
		{
			// Token: 0x060007A8 RID: 1960 RVA: 0x000219D1 File Offset: 0x0001FBD1
			get;
			// Token: 0x060007A9 RID: 1961 RVA: 0x000219D9 File Offset: 0x0001FBD9
			set;
		}
	}
}
