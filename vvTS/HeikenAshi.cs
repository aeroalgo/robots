using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200002C RID: 44
	[HandlerCategory("vvIndicators"), HandlerName("Heiken Ashi")]
	public class HeikenAshi : IBar2BarHandler, IOneSourceHandler, IStreamHandler, IHandler, ISecurityReturns, ISecurityInputs, IContextUses
	{
		// Token: 0x0600018E RID: 398 RVA: 0x000078BC File Offset: 0x00005ABC
		public ISecurity Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> openPrices = src.get_OpenPrices();
			IList<Bar> list = new List<Bar>(count);
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (i < 2)
				{
					array4[i] = openPrices[i];
					array[i] = closePrices[i];
					array3[i] = lowPrices[i];
					array2[i] = highPrices[i];
				}
				else
				{
					array[i] = (closePrices[i] + openPrices[i] + lowPrices[i] + highPrices[i]) / 4.0;
					array4[i] = (array4[i - 1] + array[i - 1]) / 2.0;
					array3[i] = Math.Min(lowPrices[i], Math.Min(array4[i], array[i]));
					array2[i] = Math.Max(highPrices[i], Math.Max(array4[i], array[i]));
				}
				Bar item = new Bar(src.get_Bars()[i].get_Color(), src.get_Bars()[i].get_Date(), array4[i], array2[i], array3[i], array[i], src.get_Bars()[i].get_Volume());
				list.Add(item);
			}
			return src.CloneAndReplaceBars(list);
		}

		// Token: 0x17000084 RID: 132
		public IContext Context
		{
			// Token: 0x0600018F RID: 399 RVA: 0x00007A64 File Offset: 0x00005C64
			get;
			// Token: 0x06000190 RID: 400 RVA: 0x00007A6C File Offset: 0x00005C6C
			set;
		}
	}
}
