using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003E RID: 62
	[HandlerCategory("vvIndicators"), HandlerName("PatternFinder v1")]
	public class PatternFinder : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000238 RID: 568 RVA: 0x0000AB9B File Offset: 0x00008D9B
		public IList<double> Execute(ISecurity src)
		{
			return PatternFinder.GenPatternFinder(src, this.PatternNo);
		}

		// Token: 0x06000237 RID: 567 RVA: 0x0000A404 File Offset: 0x00008604
		public static IList<double> GenPatternFinder(ISecurity src, int _pattern)
		{
			int count = src.get_Bars().Count;
			int num = 10;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> openPrices = src.get_OpenPrices();
			IList<double> volumes = src.get_Volumes();
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = 0.0;
				if (i >= num)
				{
					double num2 = openPrices[i];
					double num3 = openPrices[i - 1];
					double num4 = openPrices[i - 2];
					double num5 = highPrices[i];
					double num6 = highPrices[i - 1];
					double num7 = highPrices[i - 2];
					double num8 = lowPrices[i];
					double num9 = lowPrices[i - 1];
					double num10 = lowPrices[i - 2];
					double num11 = closePrices[i];
					double num12 = closePrices[i - 1];
					double num13 = closePrices[i - 2];
					double arg_F2_0 = volumes[i];
					double arg_FE_0 = volumes[i - 1];
					double arg_10A_0 = volumes[i - 2];
					bool flag = false;
					switch (_pattern)
					{
					case 0:
						if (num12 > num3 && num2 > num11 && num2 >= num12 && num3 >= num11 && num2 - num11 > num12 - num3)
						{
							flag = true;
						}
						break;
					case 1:
						if (num3 > num12 && num11 > num2 && num11 >= num3 && num12 >= num2 && num11 - num2 > num3 - num12)
						{
							flag = true;
						}
						break;
					case 2:
						if (num13 > num4 && num3 > num12 && num3 >= num13 && num4 >= num12 && num3 - num12 > num13 - num4 && num2 > num11 && num11 < num12)
						{
							flag = true;
						}
						break;
					case 3:
						if (num4 > num13 && num12 > num3 && num12 >= num4 && num13 >= num3 && num12 - num3 > num4 - num13 && num11 > num2 && num11 > num12)
						{
							flag = true;
						}
						break;
					case 4:
						if (num12 > num3 && (num12 + num3) / 2.0 > num11 && num2 > num11 && num2 > num12 && num11 > num3 && (num2 - num11) / (0.001 + (num5 - num8)) > 0.6)
						{
							flag = true;
						}
						break;
					case 5:
						if (num12 < num3 && (num3 + num12) / 2.0 < num11 && num2 < num11 && num2 < num12 && num11 < num3 && (num11 - num2) / (0.001 + (num5 - num8)) > 0.6)
						{
							flag = true;
						}
						break;
					case 6:
						if (num13 > num4 && (num13 - num4) / (0.001 + num7 - num10) > 0.6 && num13 < num3 && num12 > num3 && num6 - num9 > 3.0 * (num12 - num3) && num2 > num11 && num2 < num3)
						{
							flag = true;
						}
						break;
					case 7:
						if (num4 > num13 && (num4 - num13) / (0.001 + num7 - num10) > 0.6 && num13 > num3 && num3 > num12 && num6 - num9 > 3.0 * (num12 - num3) && num11 > num2 && num2 > num3)
						{
							flag = true;
						}
						break;
					case 8:
						if (num13 > num4 && (num13 - num4) / (0.001 + num7 - num10) > 0.6 && num13 < num3 && num12 > num3 && num6 - num9 > 3.0 * (num12 - num3) && num2 > num11 && num2 < num3)
						{
							flag = true;
						}
						break;
					case 9:
						if (num13 < num4 && (num4 - num13) / (0.001 + num7 - num10) > 0.6 && num13 > num3 && num3 > num12 && num6 - num9 > 3.0 * (num12 - num3) && num11 > num2 && num2 > num3)
						{
							flag = true;
						}
						break;
					case 10:
						if (num3 > num12 && num11 > num2 && num11 <= num3 && num12 <= num2 && num11 - num2 < num3 - num12)
						{
							flag = true;
						}
						break;
					case 11:
						if (num12 > num3 && num2 > num11 && num2 <= num12 && num3 <= num11 && num2 - num11 < num12 - num3)
						{
							flag = true;
						}
						break;
					case 12:
						if (num2 > num11 * 1.01 && num3 > num12 * 1.01 && num4 > num13 * 1.01 && num11 < num12 && num12 < num13 && num2 > num12 && num2 < num3 && num3 > num13 && num3 < num4 && (num11 - num8) / (num5 - num8) < 0.2 && (num12 - num9) / (num6 - num9) < 0.2 && (num13 - num10) / (num7 - num10) < 0.2)
						{
							flag = true;
						}
						break;
					case 13:
						if (num11 > num2 * 1.01 && num12 > num3 * 1.01 && num13 > num4 * 1.01 && num11 > num12 && num12 > num13 && num2 < num12 && num2 > num3 && num3 < num13 && num3 > num4 && (num5 - num11) / (num5 - num8) < 0.2 && (num6 - num12) / (num6 - num9) < 0.2 && (num7 - num13) / (num7 - num10) < 0.2)
						{
							flag = true;
						}
						break;
					case 14:
						if (num13 > num4 && num3 > num12 && num3 <= num13 && num4 <= num12 && num3 - num12 < num13 - num4 && num2 > num11 && num11 < num12 && num2 < num3)
						{
							flag = true;
						}
						break;
					case 15:
						if (num4 > num13 && num12 > num3 && num12 <= num4 && num13 <= num3 && num12 - num3 < num4 - num13 && num11 > num2 && num11 > num12 && num2 > num3)
						{
							flag = true;
						}
						break;
					default:
						flag = false;
						break;
					}
					if (flag)
					{
						array[i] = 1.0;
					}
				}
			}
			return array;
		}

		// Token: 0x170000C0 RID: 192
		public IContext Context
		{
			// Token: 0x06000239 RID: 569 RVA: 0x0000ABA9 File Offset: 0x00008DA9
			get;
			// Token: 0x0600023A RID: 570 RVA: 0x0000ABB1 File Offset: 0x00008DB1
			set;
		}

		// Token: 0x170000BF RID: 191
		[HandlerParameter(true, "0", Min = "0", Max = "15", Step = "1", Name = "Паттерн:\n0-медв.поглощение\n1-быч.поглощение\n2-подтвержд. медв. поглощение\n3-подтвержд. бычье поглощение\n4-завеса из темных облаков\n5-просвет в облаках\n6-вечерняя звезда доджи\n7-утренняя звезда доджи\n8-вечерняя звезда\n9-утренняя звезда\n10-бычья харами\n11-медвежья харами\n12-три черных вороны\n13-три белых солдата\n14-Three Inside Down\n15-Three Inside Up")]
		public int PatternNo
		{
			// Token: 0x06000235 RID: 565 RVA: 0x0000A3F2 File Offset: 0x000085F2
			get;
			// Token: 0x06000236 RID: 566 RVA: 0x0000A3FA File Offset: 0x000085FA
			set;
		}
	}
}
