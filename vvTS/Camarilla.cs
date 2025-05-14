using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000016 RID: 22
	[HandlerCategory("vvIndicators"), HandlerName("Camarilla")]
	public class Camarilla : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000B4 RID: 180 RVA: 0x000043B8 File Offset: 0x000025B8
		public IList<double> Execute(ISecurity src)
		{
			return Camarilla.GenCamarilla(src, this.Context, this.Output, this.G_coef);
		}

		// Token: 0x060000B3 RID: 179 RVA: 0x0000411C File Offset: 0x0000231C
		public static IList<double> GenCamarilla(ISecurity sec, IContext ctx, int _Output, double _G_coef)
		{
			int count = sec.get_Bars().Count;
			IList<double> arg_2D_0 = sec.get_ClosePrices();
			IList<double> arg_3A_0 = sec.get_OpenPrices();
			double[] array = new double[count];
			IList<double> result = array;
			double num = 0.091667;
			double num2 = 0.183333;
			double num3 = 0.275;
			double num4 = 0.55;
			IList<double> data = ctx.GetData("prevclose", new string[]
			{
				sec.get_CacheName()
			}, () => SessionClose.GetSessionClose(sec, 1));
			IList<double> data2 = ctx.GetData("prevhigh", new string[]
			{
				sec.get_CacheName()
			}, () => SessionHigh.GetSessionHigh(sec, 1));
			IList<double> data3 = ctx.GetData("prevlow", new string[]
			{
				sec.get_CacheName()
			}, () => SessionLow.GetSessionLow(sec, 1));
			for (int i = 1; i < count; i++)
			{
				double num5 = data[i];
				double num6 = data2[i];
				double num7 = data3[i];
				double num8 = (num5 + num6 + num7) / 3.0;
				double num9 = num6 / num7 * num5;
				double num10 = (num6 - num7) * num4 + num5;
				double num11 = num5 - (num9 - num5);
				double num12 = num5 - (num6 - num7) * num4;
				switch (_Output)
				{
				case 0:
					array[i] = num8;
					break;
				case 1:
					array[i] = num5 - (num6 - num7) * num;
					break;
				case 2:
					array[i] = num5 - (num6 - num7) * num2;
					break;
				case 3:
					array[i] = num5 - (num6 - num7) * num3;
					break;
				case 4:
					array[i] = num12;
					break;
				case 5:
					array[i] = num11;
					break;
				case 6:
					array[i] = (num6 - num7) * num + num5;
					break;
				case 7:
					array[i] = (num6 - num7) * num2 + num5;
					break;
				case 8:
					array[i] = (num6 - num7) * num3 + num5;
					break;
				case 9:
					array[i] = num10;
					break;
				case 10:
					array[i] = num9;
					break;
				case 11:
					array[i] = (num9 - num10) * _G_coef + num10;
					break;
				case 12:
					array[i] = num12 - (num12 - num11) * _G_coef;
					break;
				default:
					array[i] = (array[i] = num8);
					break;
				}
			}
			return result;
		}

		// Token: 0x1700003A RID: 58
		public IContext Context
		{
			// Token: 0x060000B5 RID: 181 RVA: 0x000043D2 File Offset: 0x000025D2
			get;
			// Token: 0x060000B6 RID: 182 RVA: 0x000043DA File Offset: 0x000025DA
			set;
		}

		// Token: 0x17000039 RID: 57
		[HandlerParameter(true, "0.2", Min = "0", Max = "1", Step = "0.1")]
		public double G_coef
		{
			// Token: 0x060000B1 RID: 177 RVA: 0x000040D9 File Offset: 0x000022D9
			get;
			// Token: 0x060000B2 RID: 178 RVA: 0x000040E1 File Offset: 0x000022E1
			set;
		}

		// Token: 0x17000038 RID: 56
		[HandlerParameter(true, "0", Min = "0", Max = "30", Step = "1", Name = "Output: 0-Pivot\n1-L1,2-L2,3-L3,4-L4,5-L5\n6-H1,7-H2,8-H3,9-H4,10-H5\n11-G1,12-G2")]
		public int Output
		{
			// Token: 0x060000AF RID: 175 RVA: 0x000040C8 File Offset: 0x000022C8
			get;
			// Token: 0x060000B0 RID: 176 RVA: 0x000040D0 File Offset: 0x000022D0
			set;
		}
	}
}
