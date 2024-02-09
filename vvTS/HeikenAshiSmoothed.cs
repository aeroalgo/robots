using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200002D RID: 45
	[HandlerCategory("vvIndicators"), HandlerName("Heiken Ashi smoothed")]
	public class HeikenAshiSmoothed : IBar2BarHandler, IOneSourceHandler, IStreamHandler, IHandler, ISecurityReturns, ISecurityInputs, IContextUses
	{
		// Token: 0x06000198 RID: 408 RVA: 0x00007B18 File Offset: 0x00005D18
		public ISecurity Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> C = src.get_ClosePrices();
			IList<double> H = src.get_HighPrices();
			IList<double> L = src.get_LowPrices();
			IList<double> O = src.get_OpenPrices();
			IList<Bar> list = new List<Bar>(count);
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			IList<double> data = this.Context.GetData("lwma", new string[]
			{
				this.MaPeriod.ToString(),
				src.get_OpenPrices().GetHashCode().ToString()
			}, () => LWMA.GenWMA(O, this.MaPeriod));
			IList<double> data2 = this.Context.GetData("lwma", new string[]
			{
				this.MaPeriod.ToString(),
				src.get_ClosePrices().GetHashCode().ToString()
			}, () => LWMA.GenWMA(C, this.MaPeriod));
			IList<double> data3 = this.Context.GetData("lwma", new string[]
			{
				this.MaPeriod.ToString(),
				src.get_HighPrices().GetHashCode().ToString()
			}, () => LWMA.GenWMA(H, this.MaPeriod));
			IList<double> data4 = this.Context.GetData("lwma", new string[]
			{
				this.MaPeriod.ToString(),
				src.get_LowPrices().GetHashCode().ToString()
			}, () => LWMA.GenWMA(L, this.MaPeriod));
			for (int i = 0; i < count; i++)
			{
				if (i < 2)
				{
					array4[i] = O[i];
					array[i] = C[i];
					array3[i] = L[i];
					array2[i] = H[i];
				}
				else
				{
					array[i] = (data2[i] + data[i] + data4[i] + data3[i]) / 4.0;
					array4[i] = (array4[i - 1] + array[i - 1]) / 2.0;
					array3[i] = Math.Min(data4[i], Math.Min(array4[i], array[i]));
					array2[i] = Math.Max(data3[i], Math.Max(array4[i], array[i]));
				}
			}
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			double[] array8 = new double[count];
			for (int j = 1; j < count; j++)
			{
				array5[j] = vvSeries.iMA(array4, array5, this.MaMethod, this.MaPeriod2, j, 0.0, 0.0);
				array6[j] = vvSeries.iMA(array, array6, this.MaMethod, this.MaPeriod2, j, 0.0, 0.0);
				array8[j] = vvSeries.iMA(array3, array8, this.MaMethod, this.MaPeriod2, j, 0.0, 0.0);
				array7[j] = vvSeries.iMA(array2, array7, this.MaMethod, this.MaPeriod2, j, 0.0, 0.0);
			}
			for (int k = 0; k < count; k++)
			{
				Bar item = new Bar(src.get_Bars()[k].get_Color(), src.get_Bars()[k].get_Date(), array5[k], array7[k], array8[k], array6[k], src.get_Bars()[k].get_Volume());
				list.Add(item);
			}
			return src.CloneAndReplaceBars(list);
		}

		// Token: 0x17000088 RID: 136
		public IContext Context
		{
			// Token: 0x06000199 RID: 409 RVA: 0x00007F42 File Offset: 0x00006142
			get;
			// Token: 0x0600019A RID: 410 RVA: 0x00007F4A File Offset: 0x0000614A
			set;
		}

		// Token: 0x17000085 RID: 133
		[HandlerParameter(true, "3", Min = "0", Max = "4", Step = "1")]
		public int MaMethod
		{
			// Token: 0x06000192 RID: 402 RVA: 0x00007A7D File Offset: 0x00005C7D
			get;
			// Token: 0x06000193 RID: 403 RVA: 0x00007A85 File Offset: 0x00005C85
			set;
		}

		// Token: 0x17000086 RID: 134
		[HandlerParameter(true, "6", Min = "2", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x06000194 RID: 404 RVA: 0x00007A8E File Offset: 0x00005C8E
			get;
			// Token: 0x06000195 RID: 405 RVA: 0x00007A96 File Offset: 0x00005C96
			set;
		}

		// Token: 0x17000087 RID: 135
		[HandlerParameter(true, "2", Min = "2", Max = "20", Step = "1")]
		public int MaPeriod2
		{
			// Token: 0x06000196 RID: 406 RVA: 0x00007A9F File Offset: 0x00005C9F
			get;
			// Token: 0x06000197 RID: 407 RVA: 0x00007AA7 File Offset: 0x00005CA7
			set;
		}
	}
}
